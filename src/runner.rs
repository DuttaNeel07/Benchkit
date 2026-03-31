use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};

use crate::output;
use crate::stats::{Samples, Summary};

#[derive(Args, Debug, Clone)]
pub struct RunArgs {
    /// The shell command to benchmark (passed to $SHELL -c)
    #[arg(value_name = "CMD")]
    pub command: String,

    /// Number of timed measurement runs
    #[arg(short = 'n', long, default_value_t = 10, value_name = "N")]
    pub runs: u32,

    /// Number of warmup runs (not counted in results)
    #[arg(short = 'w', long, default_value_t = 3, value_name = "N")]
    pub warmup: u32,

    /// Allow non-zero exit codes (by default any failure aborts early)
    #[arg(long)]
    pub ignore_failure: bool,

    /// Emit results as JSON instead of a table
    #[arg(long)]
    pub json: bool,

    /// Label shown in output (defaults to the command string)
    #[arg(short = 'l', long, value_name = "LABEL")]
    pub label: Option<String>,

    /// Show per-run timings in addition to the summary table
    #[arg(long)]
    pub show_runs: bool,
}

/// Execute one invocation of `cmd` via the shell and return wall-clock time.
/// Stdout/stderr are suppressed so they don't pollute terminal output.
fn time_one(cmd: &str, ignore_failure: bool) -> Result<Duration> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    let t0 = Instant::now();
    let status = Command::new(&shell)
        .args(["-c", cmd])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .with_context(|| format!("failed to spawn `{shell}`"))?;

    let elapsed = t0.elapsed();

    if !ignore_failure && !status.success() {
        let code = status.code().unwrap_or(-1);
        bail!("command exited with code {code} — use --ignore-failure to skip this check");
    }

    Ok(elapsed)
}

fn make_progress_bar(total: u32, prefix: &str) -> ProgressBar {
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{prefix:.bold.dim}  [{bar:30.cyan/blue}]  {pos}/{len}  {elapsed_precise}",
        )
        .unwrap()
        .progress_chars("=> "),
    );
    pb.set_prefix(prefix.to_string());
    pb
}

/// Run the full benchmark cycle for one command and return its summary.
pub fn bench_one(args: &RunArgs) -> Result<(Summary, Vec<f64>)> {
    let label = args
        .label
        .clone()
        .unwrap_or_else(|| args.command.clone());

    // ── warmup ───────────────────────────────────────────────────────────────
    if args.warmup > 0 {
        let pb = make_progress_bar(args.warmup, "warmup ");
        for _ in 0..args.warmup {
            time_one(&args.command, args.ignore_failure)?;
            pb.inc(1);
        }
        pb.finish_and_clear();
    }

    // ── timed runs ───────────────────────────────────────────────────────────
    let pb = make_progress_bar(args.runs, &format!("  {label}"));
    let mut timings: Vec<f64> = Vec::with_capacity(args.runs as usize);

    for _ in 0..args.runs {
        let d = time_one(&args.command, args.ignore_failure)?;
        timings.push(d.as_secs_f64());
        pb.inc(1);
    }
    pb.finish_and_clear();

    let summary = Samples::new(timings.clone()).summarise();
    Ok((summary, timings))
}

/// Entry point for the `run` subcommand.
pub fn run(args: RunArgs) -> Result<()> {
    validate(&args)?;

    let label = args
        .label
        .clone()
        .unwrap_or_else(|| args.command.clone());

    eprintln!();
    eprintln!("  command : {}", args.command);
    eprintln!("  runs    : {}  (+ {} warmup)", args.runs, args.warmup);
    eprintln!();

    let (summary, timings) = bench_one(&args)?;

    if args.show_runs {
        output::print_runs(&timings);
    }

    if args.json {
        output::print_json_single(&label, &summary)?;
    } else {
        output::print_table_single(&label, &summary);
    }

    Ok(())
}

fn validate(args: &RunArgs) -> Result<()> {
    if args.runs == 0 {
        bail!("--runs must be at least 1");
    }
    if args.command.trim().is_empty() {
        bail!("command cannot be empty");
    }
    Ok(())
}