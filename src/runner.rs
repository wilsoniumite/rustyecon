use crate::output::checkpoint::{self, SaveFormat};
use crate::output::telemetry::Telemetry;
use crate::scenario::EventSchedule;
use crate::state::{GameData, SimState};
use crate::systems::run_tick;
use std::path::PathBuf;

pub struct RunConfig {
    pub ticks: u64,
    /// Save binary checkpoint every N ticks. 0 = disabled.
    pub checkpoint_every: u64,
    /// Save human-readable RON every N ticks. 0 = disabled.
    pub human_save_every: u64,
    pub output_dir: PathBuf,
    /// Record per-tick prices and inventories to CSV in output_dir.
    pub record: bool,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            ticks: 1000,
            checkpoint_every: 52,
            human_save_every: 0,
            output_dir: PathBuf::from("output"),
            record: false,
        }
    }
}

pub struct SimRunner {
    pub state: SimState,
    pub game_data: GameData,
    pub events: EventSchedule,
    pub config: RunConfig,
}

impl SimRunner {
    pub fn new(
        state: SimState,
        game_data: GameData,
        events: EventSchedule,
        config: RunConfig,
    ) -> Self {
        Self { state, game_data, events, config }
    }

    pub fn run(&mut self) {
        let num_nodes = self.game_data.num_nodes();
        let num_goods = self.game_data.num_goods();
        let mut telemetry = self.config.record.then(|| Telemetry::new(num_nodes, num_goods));

        // Record tick-0 starting state so the inflation baseline is the initial prices.
        if let Some(t) = &mut telemetry {
            t.record(&self.state);
        }

        let total = self.config.ticks;
        for _ in 0..total {
            run_tick(&mut self.state, &self.game_data, &self.events);
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
