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

    /// Run the certification batteries, print the certificate verdict-first,
    /// persist it, and exit nonzero if any battery fails
    #[arg(long)]
    certify: bool,

    /// Directory certificates are written to (verdicts are committed to the repo)
    #[arg(long, default_value = "results")]
    results: PathBuf,
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
        certify: args.certify,
        results_dir: args.results,
    };
    let results_dir = config.results_dir.clone();

    let mut runner = SimRunner::new(scenario.state, scenario.game_data, scenario.events, config)
        .with_scenario_dir(&args.scenario)
        .with_criteria(scenario.criteria);

    if !args.certify {
        println!("running {} ticks from scenario '{}'", args.ticks, args.scenario.display());
    }
    let certificate = runner.run();

    // Verdict first: the batteries are printed before any result is read, so a
    // failed run cannot be skimmed as if it were a result (METHODOLOGY R5).
    if let Some(cert) = certificate {
        print!("{}", cert.render());
        match cert.persist(&results_dir) {
            Ok(path) => println!("certificate: {}", path.display()),
            Err(e) => {
                eprintln!("certificate write failed: {e}");
                std::process::exit(2);
            }
        }
        if !cert.passed() {
            std::process::exit(1);
        }
    } else {
        println!("done — final tick: {}", runner.tick());
    }
}
