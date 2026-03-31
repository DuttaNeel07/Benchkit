use anyhow::Result;
use colored::Colorize;

use crate::compare::CmpResult;
use crate::stats::Summary;

// ─────────────────────────────────────────────────────────────────────────────
// helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Format seconds into a human-readable string with adaptive units.
/// Sub-millisecond → µs, sub-second → ms, otherwise → s.
fn fmt_duration(secs: f64) -> String {
    if secs < 0.001 {
        format!("{:.2} µs", secs * 1_000_000.0)
    } else if secs < 1.0 {
        format!("{:.2} ms", secs * 1_000.0)
    } else {
        format!("{:.4} s", secs)
    }
}

/// Right-pad a plain string to `width` chars, then apply color.
/// We have to pad *before* colorizing because ANSI escape codes inflate
/// the byte length and break {:>N$} alignment entirely.
fn rpad(s: &str, width: usize) -> String {
    format!("{:>width$}", s, width = width)
}

fn lpad(s: &str, width: usize) -> String {
    format!("{:<width$}", s, width = width)
}

fn separator(width: usize) -> String {
    "─".repeat(width)
}

// ─────────────────────────────────────────────────────────────────────────────
// single-command table
// ─────────────────────────────────────────────────────────────────────────────

pub fn print_table_single(label: &str, s: &Summary) {
    const COL: usize = 14;
    const SEP: usize = 62;

    println!();
    println!("  {}", label.bold().underline());
    println!("  {}", separator(SEP));

    let row = |name: &str, val: f64, extra: Option<String>| {
        let val_s = fmt_duration(val);
        match extra {
            Some(e) => println!(
                "  {}  {}    {}",
                lpad(name, 12).dimmed(),
                rpad(&val_s, COL).cyan().bold(),
                e.dimmed(),
            ),
            None => println!(
                "  {}  {}",
                lpad(name, 12).dimmed(),
                rpad(&val_s, COL).cyan().bold(),
            ),
        }
    };

    row("mean", s.mean, Some(format!("± {}", fmt_duration(s.stddev))));
    row("min", s.min, None);
    row("max", s.max, None);
    row("p50", s.p50, None);
    row("p95", s.p95, None);
    row("p99", s.p99, None);

    println!("  {}", separator(SEP));
    println!(
        "  {}  {}",
        lpad("runs", 12).dimmed(),
        rpad(&s.n.to_string(), COL).yellow(),
    );
    println!(
        "  {}  {}",
        lpad("rsd", 12).dimmed(),
        rpad(&format!("{:.1}%", s.rsd_pct()), COL).yellow(),
    );

    if s.rsd_pct() > 10.0 {
        println!();
        println!(
            "  {} high variance ({:.1}% rsd) — consider more warmup or --runs",
            "warn:".yellow().bold(),
            s.rsd_pct(),
        );
    }

    println!();
}

// ─────────────────────────────────────────────────────────────────────────────
// comparison table
// ─────────────────────────────────────────────────────────────────────────────

pub fn print_table_cmp(r: &CmpResult) {
    const COL: usize = 14;
    const SEP: usize = 80;

    println!();
    println!("  {}", "Benchmark comparison".bold().underline());
    println!("  {}", separator(SEP));

    // header — plain strings padded before coloring
    println!(
        "  {}  {}  {}  {}  {}",
        lpad("", 12),
        rpad(&r.baseline.label, COL).bold(),
        rpad(&r.challenger.label, COL).bold(),
        rpad("diff", COL).dimmed(),
        rpad("ratio", COL).dimmed(),
    );
    println!("  {}", separator(SEP));

    let stat_row = |name: &str, va: f64, vb: f64| {
        let diff = vb - va;
        let ratio = if va != 0.0 { vb / va } else { 0.0 };

        let diff_raw = if diff < 0.0 {
            format!("-{}", fmt_duration(diff.abs()))
        } else {
            format!("+{}", fmt_duration(diff))
        };
        let ratio_raw = format!("{:.3}×", ratio);

        let diff_col = rpad(&diff_raw, COL);
        let ratio_col = rpad(&ratio_raw, COL);

        println!(
            "  {}  {}  {}  {}  {}",
            lpad(name, 12).dimmed(),
            rpad(&fmt_duration(va), COL).cyan().bold(),
            rpad(&fmt_duration(vb), COL).cyan().bold(),
            if diff < 0.0 { diff_col.green() } else { diff_col.red() },
            if ratio < 1.0 { ratio_col.green() } else { ratio_col.red() },
        );
    };

    let a = &r.baseline.summary;
    let b = &r.challenger.summary;

    stat_row("mean", a.mean, b.mean);
    stat_row("stddev", a.stddev, b.stddev);
    stat_row("min", a.min, b.min);
    stat_row("max", a.max, b.max);
    stat_row("p50", a.p50, b.p50);
    stat_row("p95", a.p95, b.p95);
    stat_row("p99", a.p99, b.p99);

    println!("  {}", separator(SEP));

    let speedup = if r.speedup_mean >= 1.0 {
        r.speedup_mean
    } else {
        1.0 / r.speedup_mean
    };
    let verdict = format!("{} is {:.2}× faster on average", r.faster.bold(), speedup);
    println!("  verdict : {}", verdict.green());
    println!("  runs    : {}  each", a.n.to_string().yellow());
    println!();
}

// ─────────────────────────────────────────────────────────────────────────────
// per-run listing
// ─────────────────────────────────────────────────────────────────────────────

pub fn print_runs(timings: &[f64]) {
    println!("  {}", "per-run timings".dimmed().underline());
    for (i, t) in timings.iter().enumerate() {
        println!("  {:>3}.  {}", i + 1, fmt_duration(*t).cyan());
    }
    println!();
}

// ─────────────────────────────────────────────────────────────────────────────
// JSON
// ─────────────────────────────────────────────────────────────────────────────

pub fn print_json_single(label: &str, s: &Summary) -> Result<()> {
    #[derive(serde::Serialize)]
    struct Payload<'a> {
        label: &'a str,
        #[serde(flatten)]
        summary: &'a Summary,
    }
    let p = Payload { label, summary: s };
    println!("{}", serde_json::to_string_pretty(&p)?);
    Ok(())
}

pub fn print_json_cmp(r: &CmpResult) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(r)?);
    Ok(())
}