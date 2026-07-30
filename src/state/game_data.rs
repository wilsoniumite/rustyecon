use crate::types::{
    channel::ChannelDef,
    good::GoodDef,
    ids::{ChannelId, GoodId, MarketNodeId, RecipeId, RegionId},
    market_node::MarketNodeDef,
    need_category::NeedCategory,
    recipe::RecipeDef,
};
use serde::{Deserialize, Serialize};

/// Demand quantities for each need category at one integer wealth tier.
/// `qty_per_pop[cat_idx]` = units demanded per person per tick from that category.
/// Categories not listed for this tier have qty = 0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WealthLevel {
    pub tier: u8,
    pub qty_per_pop: Vec<f64>, // length == game_data.need_categories.len()
}

/// The desk kernel's dials ([kernel.md](../../docs/architecture/kernel.md),
/// "Parameters"). Registered in `game_data.ron`, one block per scenario.
///
/// METHODOLOGY R2: no behavioural constant lives in code. There is deliberately
/// **no `Default` impl** — a scenario that omits the block fails to load rather
/// than silently inheriting whatever the source file happened to say, which is
/// the failure mode R2 exists to prevent. Phase 5 sweeps these to map the
/// collapse phase, and a value that cannot be swept is not registered.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KernelParams {
    /// Activation stagger period, in ticks. A desk acts when
    /// `(tick + desk_id) % s == 0`, so each desk moves once per `s` ticks and
    /// the population is spread evenly across the phase. Stagger divides the
    /// effective step size and desynchronises cobweb spirals without an RNG.
    pub s: u64,
    /// Multiplicative scale step up, per activation.
    pub eta_up: f64,
    /// Multiplicative scale step down, per activation. The `eta_up/eta_dn`
    /// ratio is the phase-transition dial (Gualdi et al. 2015), so it is one
    /// exposed parameter rather than a dozen implicit ones.
    pub eta_dn: f64,
    /// Dead-band half-width on the pressure signal σ: no scale response inside
    /// ±`dead`. Kills limit-cycle chatter at equilibrium.
    pub dead: f64,
    /// Output buffer band, in activations of throughput. Rule 1 posts stock
    /// above `b_out · scale · qty_out`, so this is the low-pass filter between
    /// a demand shock and a scale response.
    pub b_out: f64,
    /// Cash band, in activations of outlay. Rule 3 holds
    /// `b_cash · outlay(scale) / s` back as reserve, which is also what anchors
    /// the price *level*: at rest `M ≈ Σ b_cash · outlay / s`.
    pub b_cash: f64,
    /// Within-category consumption logit sensitivity. Inert wherever every need
    /// category has a single entry — which is the whole lr corpus — so it is
    /// registered but not exercised there.
    pub beta: f64,
    /// Scale floor as a fraction of nameplate size: `scale ∈ [epsilon·size, size]`.
    /// Nonzero so a desk that shrank to nothing can still observe a recovering
    /// market and grow back; a desk clamped to exactly zero never returns.
    pub epsilon: f64,
    /// The irrational that generates the Weyl fraction
    /// `frac(phi · (desk_id + tick/s))`, giving heterogeneous step sizes with
    /// zero RNG. Any irrational in (0,1) equidistributes; this is registered
    /// rather than hardcoded because *which* one changes the trajectory.
    pub phi: f64,
    /// EMA weight on a service desk's realized fill. Non-storable outputs have
    /// no stock to smooth them, so this plays the buffer band's role.
    pub fill_alpha: f64,
}

impl KernelParams {
    /// Whether a good's stock survives long enough for Rule 1's buffer band to
    /// act as an integrator.
    ///
    /// The band holds `b_out` activations of throughput and an activation is
    /// `s` ticks, so a lot must live at least `b_out · s` ticks or the buffer
    /// spoils before it can be drawn down — and a band that always evaporates
    /// cannot accumulate the flow error that σ is supposed to see. Goods that
    /// fail this take the service branch of Rule 1 instead. Derived from the
    /// registered parameters, so it is not a further threshold.
    pub fn storable(&self, shelf: &crate::types::good::ShelfLife) -> bool {
        match shelf {
            crate::types::good::ShelfLife::Indefinite => true,
            crate::types::good::ShelfLife::Instant => false,
            crate::types::good::ShelfLife::Ticks(n) => {
                (*n as f64) >= self.b_out * self.s as f64
            }
        }
    }

    /// Reject a parameter set that cannot mean anything, at load time.
    ///
    /// Every one of these is a value the rules divide by, take a fraction of, or
    /// index a phase with. A run that started with `s = 0` would panic deep in a
    /// modulo somewhere thousands of ticks later; failing here says which dial.
    pub fn validate(&self) -> Result<(), String> {
        let checks: [(bool, &str); 9] = [
            (self.s >= 1, "s must be at least 1 tick"),
            (self.eta_up > 0.0 && self.eta_up < 1.0, "eta_up must be in (0, 1)"),
            (self.eta_dn > 0.0 && self.eta_dn < 1.0, "eta_dn must be in (0, 1)"),
            (self.dead >= 0.0, "dead must be non-negative"),
            (self.b_out > 0.0, "b_out must be positive"),
            (self.b_cash > 0.0, "b_cash must be positive"),
            (self.beta >= 0.0, "beta must be non-negative"),
            (
                self.epsilon > 0.0 && self.epsilon <= 1.0,
                "epsilon must be in (0, 1] — a desk clamped to zero never returns",
            ),
            (
                self.fill_alpha > 0.0 && self.fill_alpha <= 1.0,
                "fill_alpha must be in (0, 1]",
            ),
        ];
        for (ok, why) in checks {
            if !ok {
                return Err(format!("kernel: {why}"));
            }
        }
        // A rational phi makes the Weyl sequence periodic, which reintroduces
        // exactly the synchronised updates stagger exists to break up. This
        // cannot test irrationality; it rejects the values that are obviously
        // not, and the comment is the rest of the guard.
        if !(self.phi > 0.0 && self.phi < 1.0) {
            return Err("kernel: phi must be in (0, 1)".into());
        }
        for denom in 1..=12u32 {
            for num in 1..denom {
                if (self.phi - num as f64 / denom as f64).abs() < 1e-12 {
                    return Err(format!(
                        "kernel: phi = {num}/{denom} is rational, so the Weyl sequence \
                         is periodic with period {denom} and the stagger stops staggering"
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Static definitions that do not change during a simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameData {
    pub goods: Vec<GoodDef>,
    pub recipes: Vec<RecipeDef>,
    /// Desk kernel dials, registered per scenario (R2).
    pub kernel: KernelParams,
    /// Substitution categories — defines what goods satisfy each need.
    #[serde(default)]
    pub need_categories: Vec<NeedCategory>,
    /// Integer wealth tiers in ascending order. Fractional wealth is interpolated.
    #[serde(default)]
    pub wealth_levels: Vec<WealthLevel>,
    pub market_nodes: Vec<MarketNodeDef>,
    #[serde(default)]
    pub channels: Vec<ChannelDef>,
    pub regions: Vec<RegionDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionDef {
    pub id: RegionId,
    pub name: String,
    pub market_node: MarketNodeId,
    #[serde(default)]
    pub position: Option<(f64, f64)>,
}

impl GameData {
    pub fn good(&self, id: GoodId) -> &GoodDef { &self.goods[id.idx()] }
    pub fn recipe(&self, id: RecipeId) -> &RecipeDef { &self.recipes[id.idx()] }
    pub fn market_node(&self, id: MarketNodeId) -> &MarketNodeDef { &self.market_nodes[id.idx()] }
    pub fn channel(&self, id: ChannelId) -> &ChannelDef { &self.channels[id.idx()] }
    pub fn region(&self, id: RegionId) -> &RegionDef { &self.regions[id.idx()] }
    pub fn num_goods(&self) -> usize { self.goods.len() }
    pub fn num_nodes(&self) -> usize { self.market_nodes.len() }

    /// Linear interpolation of qty_per_pop across integer wealth tiers.
    /// Returns a Vec indexed by need category.
    pub fn interpolated_qty(&self, wealth: f64) -> Vec<f64> {
        let n = self.need_categories.len();
        if self.wealth_levels.is_empty() || n == 0 {
            return vec![0.0; n];
        }
        let wealth = wealth.max(0.0);
        let floor_tier = wealth.floor() as u8;
        let frac = wealth.fract();

        // Find bracketing tiers; clamp at the edges.
        let lower = self.wealth_levels.iter()
            .filter(|wl| wl.tier <= floor_tier)
            .last()
            .unwrap_or(&self.wealth_levels[0]);
        let upper = self.wealth_levels.iter()
            .find(|wl| wl.tier > floor_tier)
            .unwrap_or(lower);

        (0..n).map(|i| {
            let lo = lower.qty_per_pop.get(i).copied().unwrap_or(0.0);
            let hi = upper.qty_per_pop.get(i).copied().unwrap_or(0.0);
            lo + frac * (hi - lo)
        }).collect()
    }

    /// Largest wealth tier defined. Useful for clamping.
    pub fn max_wealth_tier(&self) -> f64 {
        self.wealth_levels.last().map(|wl| wl.tier as f64).unwrap_or(0.0)
    }

    /// Weight-proportional default sub_state: one fractions-vector per category.
    pub fn default_sub_state(&self) -> Vec<Vec<f64>> {
        self.need_categories.iter()
            .map(|cat| cat.target_fractions())
            .collect()
    }
}

#[cfg(test)]
mod kernel_tests {
    use super::*;
    use crate::types::good::ShelfLife;

    /// The registered values, as they appear in every scenario's game_data.ron.
    fn params() -> KernelParams {
        KernelParams {
            s: 4,
            eta_up: 0.04,
            eta_dn: 0.05,
            dead: 0.05,
            b_out: 2.0,
            b_cash: 6.5,
            beta: 1.0,
            epsilon: 0.01,
            phi: 0.6180339887498949,
            fill_alpha: 0.25,
        }
    }

    #[test]
    fn the_registered_parameters_validate() {
        params().validate().expect("the shipped dials must load");
    }

    #[test]
    fn a_dial_that_cannot_mean_anything_is_refused_at_load() {
        let cases: Vec<(&str, Box<dyn Fn(&mut KernelParams)>)> = vec![
            ("s", Box::new(|p: &mut KernelParams| p.s = 0)),
            ("eta_up", Box::new(|p: &mut KernelParams| p.eta_up = 0.0)),
            ("eta_dn", Box::new(|p: &mut KernelParams| p.eta_dn = 1.5)),
            ("dead", Box::new(|p: &mut KernelParams| p.dead = -0.1)),
            ("b_out", Box::new(|p: &mut KernelParams| p.b_out = 0.0)),
            ("b_cash", Box::new(|p: &mut KernelParams| p.b_cash = -1.0)),
            ("beta", Box::new(|p: &mut KernelParams| p.beta = -1.0)),
            // Zero is the interesting one: a desk clamped to exactly zero scale
            // buys nothing, produces nothing and can never observe a recovery,
            // so it is gone for the rest of the run.
            ("epsilon", Box::new(|p: &mut KernelParams| p.epsilon = 0.0)),
            ("fill_alpha", Box::new(|p: &mut KernelParams| p.fill_alpha = 0.0)),
            ("phi", Box::new(|p: &mut KernelParams| p.phi = 1.0)),
        ];
        for (name, break_it) in cases {
            let mut p = params();
            break_it(&mut p);
            let err = p.validate().unwrap_err();
            assert!(err.contains(name), "{name}: message was {err:?}");
        }
    }

    #[test]
    fn a_rational_phi_is_refused_because_it_stops_the_stagger() {
        // The Weyl fraction exists to give heterogeneous, non-repeating step
        // sizes without an RNG. phi = 1/2 makes it alternate between two values
        // forever, which is synchronised updating wearing a disguise — and
        // synchronous updating of stiff dynamics oscillates as a numerical
        // artifact, which is the thing stagger is for.
        for bad in [0.5, 0.25, 1.0 / 3.0, 7.0 / 12.0] {
            let mut p = params();
            p.phi = bad;
            let err = p.validate().unwrap_err();
            assert!(err.contains("rational"), "phi={bad}: {err}");
        }
        // The golden-ratio conjugate is the worst-approximable irrational, so
        // it is the *least* nearly-periodic choice available.
        assert!(params().validate().is_ok());
    }

    #[test]
    fn storability_is_whether_the_buffer_band_outlives_its_own_drawdown() {
        let p = params(); // b_out 2.0 x s 4 = 8 ticks of band
        assert!(p.storable(&ShelfLife::Indefinite));
        assert!(!p.storable(&ShelfLife::Instant));
        assert!(!p.storable(&ShelfLife::Ticks(1)), "the corpus's `services`");
        assert!(!p.storable(&ShelfLife::Ticks(7)));
        assert!(p.storable(&ShelfLife::Ticks(8)));

        // It is derived from the dials, not a separate threshold: widen the band
        // and goods at the margin stop being storable, because the band would
        // now spoil before it could be drawn down.
        let mut wide = p;
        wide.b_out = 4.0;
        assert!(!wide.storable(&ShelfLife::Ticks(8)));
        assert!(wide.storable(&ShelfLife::Ticks(16)));
    }
}
