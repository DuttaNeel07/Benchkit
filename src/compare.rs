use anyhow::{bail, Result};
use clap::Args;
use serde::Serialize;

use crate::output;
use crate::runner::{self, RunArgs};
use crate::stats::Summary;

#[derive(Args, Debug, Clone)]
pub struct CmpArgs {
    /// First command (treated as the baseline)
    #[arg(value_name = "CMD_A")]
    pub cmd_a: String,

    /// Second command (compared against the baseline)
    #[arg(value_name = "CMD_B")]
    pub cmd_b: String,

    /// Number of timed runs per command
    #[arg(short = 'n', long, default_value_t = 10, value_name = "N")]
    pub runs: u32,

    /// Number of warmup runs per command
    #[arg(short = 'w', long, default_value_t = 3, value_name = "N")]
    pub warmup: u32,

    /// Allow non-zero exit codes from either command
    #[arg(long)]
    pub ignore_failure: bool,

    /// Label for the first command
    #[arg(long, value_name = "LABEL")]
    pub label_a: Option<String>,

    /// Label for the second command
    #[arg(long, value_name = "LABEL")]
    pub label_b: Option<String>,

    /// Emit results as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Serialize)]
pub struct CmpResult {
    pub baseline: LabelledSummary,
    pub challenger: LabelledSummary,
    pub speedup_mean: f64,
    pub speedup_min: f64,
    pub faster: String,
}

#[derive(Debug, Serialize)]
pub struct LabelledSummary {
    pub label: String,
    pub summary: Summary,
}

pub fn run(args: CmpArgs) -> Result<()> {
    if args.runs == 0 {
        bail!("--runs must be at least 1");
    }

    let label_a = args
        .label_a
        .clone()
        .unwrap_or_else(|| args.cmd_a.clone());
    let label_b = args
        .label_b
        .clone()
        .unwrap_or_else(|| args.cmd_b.clone());

    eprintln!();
    eprintln!("  baseline   : {}", args.cmd_a);
    eprintln!("  challenger : {}", args.cmd_b);
    eprintln!("  runs       : {}  (+ {} warmup each)", args.runs, args.warmup);
    eprintln!();

    // Benchmark A
    let args_a = RunArgs {
        command: args.cmd_a.clone(),
        runs: args.runs,
        warmup: args.warmup,
        ignore_failure: args.ignore_failure,
        json: false,
        label: Some(label_a.clone()),
        show_runs: false,
    };
    let (sum_a, _) = runner::bench_one(&args_a)?;

    // Benchmark B
    let args_b = RunArgs {
        command: args.cmd_b.clone(),
        runs: args.runs,
        warmup: args.warmup,
        ignore_failure: args.ignore_failure,
        json: false,
        label: Some(label_b.clone()),
        show_runs: false,
    };
    let (sum_b, _) = runner::bench_one(&args_b)?;

    // Speedup: positive means B is faster, negative means A is faster.
    // We express it as "A is X× faster/slower than B" from baseline POV.
    let speedup_mean = sum_a.mean / sum_b.mean;
    let speedup_min = sum_a.min / sum_b.min;

    let faster = if speedup_mean >= 1.0 {
        label_b.clone()
    } else {
        label_a.clone()
    };

    let result = CmpResult {
        baseline: LabelledSummary { label: label_a, summary: sum_a },
        challenger: LabelledSummary { label: label_b, summary: sum_b },
        speedup_mean,
        speedup_min,
        faster,
    };

    if args.json {
        output::print_json_cmp(&result)?;
    } else {
        output::print_table_cmp(&result);
    }

    Ok(())
}