use crate::certify::{
    certificate::{self, Battery, Certificate, RunIdentity},
    ledger::TickAudit,
    metrics::MetricsCollector,
    nan, state_hash, verdict, Criteria,
};
use crate::output::checkpoint::{self, SaveFormat};
use crate::output::telemetry::Telemetry;
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
    /// Record per-tick prices and inventories to CSV in output_dir.
    pub record: bool,
    /// Run the certification batteries and emit a run certificate.
    pub certify: bool,
    /// Where certificates are persisted. Verdicts are committed (R5).
    pub results_dir: PathBuf,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            ticks: 1000,
            checkpoint_every: 52,
            human_save_every: 0,
            output_dir: PathBuf::from("output"),
            record: false,
            certify: false,
            results_dir: PathBuf::from("results"),
        }
    }
}

/// How often the replay's state hash is compared against the live run.
///
/// The recorded deltas are replayed every tick regardless; only the hashing is
/// throttled, because serialising the whole state twice per tick dominates the
/// cost of a certified run at the 11,700-tick horizon.
const REPLAY_CHECK_EVERY: u64 = 64;

pub struct SimRunner {
    pub state: SimState,
    pub game_data: GameData,
    pub events: EventSchedule,
    pub config: RunConfig,
    /// Which scenario produced this run, for the certificate's identity.
    scenario_name: String,
    scenario_tape_sha: String,
    /// Pre-registered stability criteria, when the scenario declares them.
    criteria: Option<Criteria>,
}

impl SimRunner {
    pub fn new(
        state: SimState,
        game_data: GameData,
        events: EventSchedule,
        config: RunConfig,
    ) -> Self {
        Self {
            state,
            game_data,
            events,
            config,
            scenario_name: "unknown".into(),
            scenario_tape_sha: "0".repeat(16),
            criteria: None,
        }
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
        self
    }

    /// Run to completion. Returns a certificate when `config.certify` is set.
    pub fn run(&mut self) -> Option<Certificate> {
        let num_nodes = self.game_data.num_nodes();
        let num_goods = self.game_data.num_goods();
        let mut telemetry = self.config.record.then(|| Telemetry::new(num_nodes, num_goods));

        // Record tick-0 starting state so the inflation baseline is the initial prices.
        if let Some(t) = &mut telemetry {
            t.record(&self.state);
        }

        // Certification state, all bounded: a shadow copy of genesis that the
        // recorded delta stream is replayed onto tick by tick, so replay equality
        // is proven without ever holding the whole stream in memory.
        let mut replay = self.config.certify.then(|| self.state.clone());
        let mut replay_diverged_at: Option<u64> = None;
        let mut audit = TickAudit::default();
        let mut nan_hits: Vec<String> = Vec::new();
        let mut collector = match (self.config.certify, &self.criteria) {
            (true, Some(c)) => Some(MetricsCollector::new(&self.game_data, c)),
            _ => None,
        };

        let total = self.config.ticks;
        for _ in 0..total {
            if self.config.certify {
                let (stream, tick_audit) =
                    run_tick_certified(&mut self.state, &self.game_data, &self.events);
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
                run_tick(&mut self.state, &self.game_data, &self.events);
            }

            self.maybe_save_checkpoints();
            if let Some(t) = &mut telemetry {
                t.record(&self.state);
            }
        }

        if let Some(t) = telemetry {
            if let Err(e) = t.write_csv(&self.config.output_dir, &self.game_data) {
                eprintln!("telemetry write failed: {e}");
            }
        }

        let stability = match (&self.criteria, &collector) {
            (Some(c), Some(m)) => Some(verdict::evaluate(c, m.series())),
            _ => None,
        };

        self.config
            .certify
            .then(|| self.build_certificate(audit, replay_diverged_at, &nan_hits, stability))
    }

    fn build_certificate(
        &self,
        audit: TickAudit,
        replay_diverged_at: Option<u64>,
        nan_hits: &[String],
        stability: Option<verdict::StabilityReport>,
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
        ];

        let git = certificate::git_sha();
        let tape = self.scenario_tape_sha.clone();
        let seed = 0; // No stochastic source exists; recorded so the field is explicit.
        let scenario = self.scenario_name.clone();
        let run = certificate::run_id(&git, &tape, &scenario, self.config.ticks, seed);

        Certificate::with_stability(
            RunIdentity {
                run,
                git,
                tape_sha: tape,
                seed,
                scenario,
                ticks: self.config.ticks,
            },
            batteries,
            stability,
        )
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
