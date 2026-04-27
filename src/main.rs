use clap::Parser;
use rustyecon::{
    runner::{RunConfig, SimRunner},
    scenario::loader,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "Headless global economic simulation")]
struct Args {
    /// Scenario directory (must contain game_data.ron, starting_state.ron, events.ron)
    #[arg(default_value = "data/scenarios/minimal")]
    scenario: PathBuf,

    /// Number of ticks to simulate
    #[arg(short, long, default_value_t = 1000)]
    ticks: u64,

    /// Output directory for checkpoints and analysis files
    #[arg(short, long, default_value = "output")]
    output: PathBuf,

    /// Save binary checkpoint every N ticks (0 = disabled)
    #[arg(long, default_value_t = 0)]
    checkpoint_every: u64,

    /// Save human-readable .ron checkpoint every N ticks (0 = disabled)
    #[arg(long, default_value_t = 0)]
    human_save: u64,

    /// Record per-tick prices and inventories to prices.csv / inventories.csv in output dir
    #[arg(long)]
    record: bool,
}

fn main() {
    let args = Args::parse();

    let scenario = match loader::load(&args.scenario) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error loading scenario '{}': {e}", args.scenario.display());
            std::process::exit(1);
        }
    };

    let config = RunConfig {
        ticks: args.ticks,
        checkpoint_every: args.checkpoint_every,
        human_save_every: args.human_save,
        output_dir: args.output,
        record: args.record,
    };

    let mut runner = SimRunner::new(scenario.state, scenario.game_data, scenario.events, config);

    println!("running {} ticks from scenario '{}'", args.ticks, args.scenario.display());
    runner.run();
    println!("done — final tick: {}", runner.tick());
}
