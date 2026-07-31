use crate::certify::{
    invariants::{own_state_scan, BalanceWatch},
    certificate::{self, Battery, Certificate, RunIdentity},
    ledger::TickAudit,
    metrics::MetricsCollector,
    nan, state_hash, technology::PriceGapWatch, verdict, Criteria,
};
use crate::output::checkpoint::{self, SaveFormat};
use crate::output::manifest::{self, RunInfo};
use crate::output::telemetry::Telemetry;
use crate::kernel::AgentArm;
use crate::scenario::EventSchedule;
use crate::state::{apply_state_deltas, GameData, SimState};
use crate::systems::{run_tick, run_tick_certified};
use std::path::{Path, PathBuf};

pub struct RunConfig {
    pub ticks: u64,
    /// Save binary checkpoint every N ticks. 0 = disabled.
    pub checkpoint_every: u64,
    /// Save human-readable RON every N ticks. 0 = disabled.
    pub human_save_every: u64,
    pub output_dir: PathBuf,
    /// Write Parquet telemetry + manifest into output_dir.
    pub record: bool,
    /// Ticks between telemetry samples. 1 = every tick. Tick 0 is always taken.
    pub telemetry_every: u64,
    /// Run the certification batteries and emit a run certificate.
    pub certify: bool,
    /// Where certificates are persisted. Verdicts are committed (R5).
    pub results_dir: PathBuf,
    /// Which agent layer decides. Recorded in the certificate and folded into
    /// the run id, so two runs of one scenario under different arms cannot be
    /// mistaken for each other.
    pub agents: AgentArm,
    /// Which price rule the market uses. A/B'd like the agent arm.
    pub price_rule: crate::systems::clearing::PriceRule,
    /// Override of the tape's registered `kernel.supply_rule`.
    ///
    /// `None` — the default — means "run what the tape registered", which is the
    /// R2-clean path: the constant lives in data. The override exists because
    /// the switch has to be swept across 33 tapes for the A/B and editing 33
    /// tapes per arm is how a corpus comes to disagree with itself. It is
    /// applied to `game_data` once, at construction, so exactly one value is in
    /// force for the whole run, and the *effective* value is folded into the run
    /// identity — an overridden certificate cannot be mistaken for the tape's.
    pub supply_rule: Option<crate::state::game_data::SupplyRule>,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            ticks: 1000,
            checkpoint_every: 52,
            human_save_every: 0,
            output_dir: PathBuf::from("output"),
            record: false,
            telemetry_every: 1,
            certify: false,
            results_dir: PathBuf::from("results"),
            agents: AgentArm::Legacy,
            price_rule: crate::systems::clearing::PriceRule::Imbalance,
            supply_rule: None,
        }
    }
}

/// How often the replay's state hash is compared against the live run.
///
/// The recorded deltas are replayed every tick regardless; only the hashing is
/// throttled, because serialising the whole state twice per tick dominates the
/// cost of a certified run at the 11,700-tick horizon.
const REPLAY_CHECK_EVERY: u64 = 64;

/// How often the own-state posting invariant is probed (B6).
///
/// Each probe clones the state and runs the decision phase twice, so it is
/// sampled rather than continuous. A rule that reads foreign demand reads it
/// every tick, so any sampling interval finds it; the interval only bounds how
/// long a violation could hide, and it is instrumentation cadence rather than
/// a behavioural constant.
const OWN_STATE_CHECK_EVERY: u64 = 64;

pub struct SimRunner {
    pub state: SimState,
    pub game_data: GameData,
    pub events: EventSchedule,
    pub config: RunConfig,
    /// Which scenario produced this run, for the certificate's identity.
    scenario_name: String,
    scenario_tape_sha: String,
    /// Hash of the scenario's criteria.ron, for the telemetry manifest.
    criteria_sha: Option<String>,
    /// Pre-registered stability criteria, when the scenario declares them.
    criteria: Option<Criteria>,
    /// First telemetry write error, if any. A run whose telemetry is incomplete
    /// must not be mistakable for one whose telemetry is whole.
    telemetry_error: Option<String>,
}

impl SimRunner {
    pub fn new(
        state: SimState,
        mut game_data: GameData,
        events: EventSchedule,
        config: RunConfig,
    ) -> Self {
        // Applied here rather than at each call site so there is one place where
        // the effective dial is decided, and so `identity()` can read it back
        // off `game_data` instead of re-deriving the precedence a second time.
        if let Some(rule) = config.supply_rule {
            game_data.kernel.supply_rule = rule;
        }
        Self {
            state,
            game_data,
            events,
            config,
            scenario_name: "unknown".into(),
            scenario_tape_sha: "0".repeat(16),
            criteria_sha: None,
            criteria: None,
            telemetry_error: None,
        }
    }

    /// The first telemetry write failure, if telemetry was requested and broke.
    pub fn telemetry_error(&self) -> Option<&str> {
        self.telemetry_error.as_deref()
    }

    /// Attach the scenario's pre-registered stability criteria.
    pub fn with_criteria(mut self, criteria: Option<Criteria>) -> Self {
        self.criteria = criteria;
        self
    }

    /// Name the scenario this run came from and fingerprint its inputs, so the
    /// certificate identifies what was actually run rather than just that
    /// something was.
    pub fn with_scenario_dir(mut self, dir: &Path) -> Self {
        self.scenario_name = dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        self.scenario_tape_sha = certificate::tape_sha(dir);
        self.criteria_sha = manifest::criteria_sha(dir);
        self
    }

    /// Identity of this run: what was run, from which inputs, at which commit.
    /// Shared by the certificate and the telemetry manifest so the two artifacts
    /// of one run cannot disagree about which run they describe.
    fn identity(&self) -> RunIdentity {
        let git = certificate::git_sha();
        let tape = self.scenario_tape_sha.clone();
        let seed = 0; // No stochastic source exists; recorded so the field is explicit.
        let scenario = self.scenario_name.clone();
        // The effective supply rule is read off `game_data`, not off the config,
        // so a CLI override and a tape value are recorded the same way and a
        // swept run can never carry the tape's label.
        let agents = format!(
            "{}+{}+{}",
            self.config.agents.name(),
            self.config.price_rule.name(),
            self.game_data.kernel.supply_rule.name()
        );
        let run =
            certificate::run_id(&git, &tape, &scenario, &agents, self.config.ticks, seed);
        RunIdentity {
            run,
            git,
            tape_sha: tape,
            seed,
            scenario,
            ticks: self.config.ticks,
            agents,
        }
    }

    /// Run to completion. Returns a certificate when `config.certify` is set.
    pub fn run(&mut self) -> Option<Certificate> {
        let num_nodes = self.game_data.num_nodes();
        let num_goods = self.game_data.num_goods();
        let mut telemetry = None;
        if self.config.record {
            match Telemetry::create(&self.config.output_dir, self.config.telemetry_every) {
                Ok(t) => telemetry = Some(t),
                Err(e) => self.telemetry_error = Some(format!("could not open telemetry: {e}")),
            }
        }

        // Record the tick-0 starting state: it is the baseline every price index
        // is relative to, so a run that skipped it would rebase silently.
        if let Some(t) = &mut telemetry {
            if let Err(e) = t.record(&self.state, &self.game_data) {
                self.telemetry_error.get_or_insert(format!("tick 0: {e}"));
            }
        }

        // Certification state, all bounded: a shadow copy of genesis that the
        // recorded delta stream is replayed onto tick by tick, so replay equality
        // is proven without ever holding the whole stream in memory.
        let mut replay = self.config.certify.then(|| self.state.clone());
        let mut replay_diverged_at: Option<u64> = None;
        let mut audit = TickAudit::default();
        let mut nan_hits: Vec<String> = Vec::new();
        // kernel.md, "Price formation": two things Phase 4 must check rather
        // than assume. Both are engine checks (R3/R5), not notebook checks.
        let mut own_state_hits: Vec<String> = Vec::new();
        let mut balance = self.config.certify.then(|| BalanceWatch::new(&self.game_data));
        // B8's accumulator. Needs the criteria for two things it must not invent:
        // which good is the numeraire, and which ticks are the scored window.
        let mut price_gap = match (self.config.certify, &self.criteria) {
            (true, Some(c)) => Some(PriceGapWatch::new(&self.game_data, c)),
            _ => None,
        };
        let mut collector = match (self.config.certify, &self.criteria) {
            (true, Some(c)) => Some(MetricsCollector::new(&self.game_data, c)),
            _ => None,
        };

        let total = self.config.ticks;
        for _ in 0..total {
            if self.config.certify {
                let (stream, tick_audit) =
                    run_tick_certified(&mut self.state, &self.game_data, &self.events, self.config.agents, self.config.price_rule);
                audit.absorb(&tick_audit);

                if let (Some(r), None) = (replay.as_mut(), replay_diverged_at) {
                    // No systems run here — only the recorded deltas are applied.
                    // If anything mutated state outside the delta stream, the two
                    // states part company.
                    apply_state_deltas(r, &stream);
                    // Deltas are replayed every tick, but hashing the whole state
                    // is the expensive part, so compare periodically and always on
                    // the final tick. Divergence persists once it happens — the
                    // replay has no way back — so a periodic check still catches it.
                    let last = self.state.tick >= self.config.ticks;
                    if last || self.state.tick % REPLAY_CHECK_EVERY == 0 {
                        if state_hash(r) != state_hash(&self.state) {
                            replay_diverged_at = Some(self.state.tick);
                        }
                    }
                }

                if let Some(m) = collector.as_mut() {
                    m.record(&self.state, &self.game_data);
                    // Emit the same samples the verdict is computed from, so the
                    // telemetry and the certificate cannot tell two stories.
                    if let Some(t) = telemetry.as_mut() {
                        if let Err(e) = t.record_region_metrics(self.state.tick, m) {
                            self.telemetry_error
                                .get_or_insert(format!("tick {}: {e}", self.state.tick));
                        }
                    }
                }

                // Sampled before the tick's own decisions are overwritten by
                // the next tick's clearing: this reads the same state a decision
                // would, which is the point.
                if own_state_hits.is_empty() && self.state.tick % OWN_STATE_CHECK_EVERY == 0 {
                    own_state_hits =
                        own_state_scan(&self.state, &self.game_data, self.config.agents);
                }
                if let Some(c) = self.criteria.as_ref() {
                    // Both read the same window as the stability verdict, so a
                    // certificate cannot report a pin or a price gap measured
                    // over a span the criteria never scored.
                    if c.in_analysis(self.state.tick) {
                        if let Some(b) = balance.as_mut() {
                            b.observe(&self.state, &self.game_data);
                        }
                        if let Some(g) = price_gap.as_mut() {
                            g.observe(&self.state);
                        }
                    }
                }

                if nan_hits.is_empty() {
                    for hit in nan::scan(&self.state, num_nodes, num_goods) {
                        nan_hits.push(format!(
                            "{} at {} (tick {})",
                            hit.field, hit.index, self.state.tick
                        ));
                    }
                }
            } else {
                run_tick(&mut self.state, &self.game_data, &self.events, self.config.agents, self.config.price_rule);
            }

            self.maybe_save_checkpoints();
            if let Some(t) = &mut telemetry {
                if let Err(e) = t.record(&self.state, &self.game_data) {
                    self.telemetry_error
                        .get_or_insert(format!("tick {}: {e}", self.state.tick));
                }
            }
        }

        if let Some(t) = telemetry {
            let run = RunInfo {
                run_id: self.identity().run,
                git_sha: certificate::git_sha(),
                tape_sha: self.scenario_tape_sha.clone(),
                criteria_sha: self.criteria_sha.clone(),
                seed: 0,
                scenario: self.scenario_name.clone(),
                ticks: self.config.ticks,
            };
            // Parquet keeps its schema and row index in a footer, so failing to
            // close leaves an unreadable file rather than a short one. Record it
            // as an error instead of letting the run look successful.
            if let Err(e) = t.finish(&self.config.output_dir, &self.game_data, run) {
                self.telemetry_error.get_or_insert(format!("closing telemetry: {e}"));
            }
        }

        let stability = match (&self.criteria, &collector) {
            (Some(c), Some(m)) => Some(verdict::evaluate(c, m.series())),
            _ => None,
        };

        self.config.certify.then(|| {
            self.build_certificate(
                audit,
                replay_diverged_at,
                &nan_hits,
                stability,
                &own_state_hits,
                balance.as_ref(),
                price_gap.as_ref(),
            )
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn build_certificate(
        &self,
        audit: TickAudit,
        replay_diverged_at: Option<u64>,
        nan_hits: &[String],
        stability: Option<verdict::StabilityReport>,
        own_state_hits: &[String],
        balance: Option<&BalanceWatch>,
        price_gap: Option<&PriceGapWatch>,
    ) -> Certificate {
        let replay_ok = replay_diverged_at.is_none();
        let hash = state_hash(&self.state);

        let batteries = vec![
            // The run would have panicked on a breach, so reaching here means the
            // identity held every tick; the margin says by how much.
            Battery::new(
                "B1",
                "conservation",
                audit.max_abs_drift.is_finite(),
                format!(
                    "max drift {:.3e}, worst margin {:.1}% of tolerance",
                    audit.max_abs_drift,
                    audit.max_margin_ratio * 100.0
                ),
            ),
            // Systems take &SimState and only apply_state_deltas takes &mut, so
            // this is enforced by the type system; the replay is the running proof.
            Battery::new(
                "B2",
                "delta-only mutation (replay audit)",
                replay_ok,
                match replay_diverged_at {
                    None => "no state escaped the delta stream".into(),
                    Some(t) => format!("state diverged from replay at tick {t}"),
                },
            ),
            Battery::new(
                "B3",
                "determinism",
                replay_ok,
                format!(
                    "state hash 0x{hash:016x}, replay {}",
                    if replay_ok { "MATCH" } else { "DIVERGED" }
                ),
            ),
            Battery::new(
                "B4",
                "metrics computable",
                nan_hits.is_empty(),
                if nan_hits.is_empty() {
                    "0 NaN".into()
                } else {
                    format!("{} non-finite: {}", nan_hits.len(), nan_hits[0])
                },
            ),
            Battery::new(
                "B5",
                "clamp/shortfall events",
                audit.shortfalls == 0,
                format!("{} (total {:.3e} missing)", audit.shortfalls, audit.shortfall_qty),
            ),
            // kernel.md's invariant, stated so it can be checked: every posted
            // quantity is a function of the posting desk's own state. Checked
            // differentially — foreign market volumes are perturbed and the
            // decision phase re-run — because how a number was computed cannot
            // be read off the number.
            Battery::new(
                "B6",
                "own-state posting",
                own_state_hits.is_empty(),
                if own_state_hits.is_empty() {
                    "no posting moved on foreign demand or supply".into()
                } else {
                    format!("{} violation(s): {}", own_state_hits.len(), own_state_hits[0])
                },
            ),
            // And the other half: a settled market must reach imbalance ~ 0
            // rather than a smaller pin. A pin is a CONSTANT, not a small
            // number — the legacy defect sat at -1/6 with standard deviation
            // exactly zero for 10,100 ticks — so this tests for constancy.
            match balance {
                Some(b) => {
                    let pins = b.pinned(&self.game_data);
                    Battery::new(
                        "B7",
                        "no pinned market",
                        pins.is_empty(),
                        if pins.is_empty() {
                            format!(
                                "{} markets, median |imbalance| {:.3e}",
                                b.markets_watched(),
                                b.median_abs_imbalance()
                            )
                        } else {
                            format!("{} pinned: {}", pins.len(), pins[0])
                        },
                    )
                }
                None => Battery::new("B7", "no pinned market", true, "unscored".into()),
            },
            // The first CORRECTNESS battery. B1..B7 all ask whether the run held
            // together or stayed put; none of them asks whether the prices went
            // to the right place, and a world that settles instantly on nonsense
            // passes every one of them. B8 compares the realised relative price
            // vector against the one the tape's own recipes imply at zero profit.
            //
            // It deliberately has NO distance threshold — see
            // `PriceGapWatch::report` for the argument, which is the R6 one: the
            // numbers were already known when this was written, so any bar
            // picked now would be a bar fitted to a result.
            match price_gap {
                Some(g) => {
                    let (pass, detail) = g.report(&self.game_data);
                    Battery::new("B8", "prices match technology", pass, detail)
                }
                None => Battery::new(
                    "B8",
                    "prices match technology",
                    true,
                    "unscored: no criteria.ron registered".into(),
                ),
            },
        ];

        Certificate::with_stability(self.identity(), batteries, stability)
    }

    fn maybe_save_checkpoints(&self) {
        let tick = self.state.tick;
        if self.config.checkpoint_every > 0 && tick % self.config.checkpoint_every == 0 {
            if let Err(e) = checkpoint::save_checkpoint(
                &self.state,
                &SaveFormat::Binary,
                &self.config.output_dir,
            ) {
                eprintln!("checkpoint save failed at tick {tick}: {e}");
            }
        }

        if self.config.human_save_every > 0 && tick % self.config.human_save_every == 0 {
            if let Err(e) = checkpoint::save_checkpoint(
                &self.state,
                &SaveFormat::HumanReadable,
                &self.config.output_dir,
            ) {
                eprintln!("human-readable save failed at tick {tick}: {e}");
            }
        }
    }

    pub fn tick(&self) -> u64 {
        self.state.tick
    }
}
