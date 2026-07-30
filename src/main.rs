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

    /// Save binary checkpoint every N ticks for resume (0 = disabled). Sparse
    /// by intent: checkpoints exist to restart a run, not to be analysed.
    #[arg(long, default_value_t = 0)]
    checkpoint_every: u64,

    /// Save human-readable .ron checkpoint every N ticks (0 = disabled).
    /// For inspecting a state by eye. NOT the analysis feed — use --record,
    /// which is what tools/readers.py reads.
    #[arg(long, default_value_t = 0)]
    human_save: u64,

    /// Write Parquet telemetry (telemetry.parquet + manifest.json) to the output dir
    #[arg(long)]
    record: bool,

    /// Ticks between telemetry samples (1 = every tick). Tick 0 is always sampled.
    #[arg(long, default_value_t = 1)]
    telemetry_every: u64,

    /// Run the certification batteries, print the certificate verdict-first,
    /// persist it, and exit nonzero if any battery fails
    #[arg(long)]
    certify: bool,

    /// Directory certificates are written to (verdicts are committed to the repo)
    #[arg(long, default_value = "results")]
    results: PathBuf,

    /// Which agent layer decides: `legacy` (the three strategies plus the pop
    /// PD controller) or `kernel` (the desk kernel's three rules). Both read the
    /// same tape; PLAN Phase 4 A/Bs them and deletes the loser.
    #[arg(long, default_value = "legacy")]
    agents: rustyecon::kernel::AgentArm,
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
        telemetry_every: args.telemetry_every,
        certify: args.certify,
        results_dir: args.results,
        agents: args.agents,
    };
    let results_dir = config.results_dir.clone();

    let mut runner = SimRunner::new(scenario.state, scenario.game_data, scenario.events, config)
        .with_scenario_dir(&args.scenario)
        .with_criteria(scenario.criteria);

    if !args.certify {
        println!("running {} ticks from scenario '{}'", args.ticks, args.scenario.display());
    }
    let certificate = runner.run();
    // A run whose telemetry broke mid-write must not be analysed as if it were
    // whole; the analysis would silently study a prefix.
    let telemetry_failed = match runner.telemetry_error() {
        Some(e) => {
            eprintln!("telemetry failed: {e}");
            true
        }
        None => false,
    };

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
    if telemetry_failed {
        std::process::exit(3);
    }
}
