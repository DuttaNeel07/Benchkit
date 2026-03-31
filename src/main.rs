mod runner;
mod stats;
mod output;
mod compare;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// benchkit (bk) — a no-nonsense command benchmarking tool.
///
/// Runs a shell command N times, collects timing samples, and reports
/// mean / stddev / min / max / p50 / p95 / p99. Supports warmup runs,
/// JSON export, and head-to-head comparison of two commands.
#[derive(Parser, Debug)]
#[command(
    name = "bk",
    version,
    about = "Measure and compare shell command performance",
    long_about = None,
    arg_required_else_help = true,
)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Benchmark a single command
    Run(runner::RunArgs),

    /// Compare two commands and show the speedup ratio
    Cmp(compare::CmpArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Command::Run(args) => runner::run(args),
        Command::Cmp(args) => compare::run(args),
    }
}