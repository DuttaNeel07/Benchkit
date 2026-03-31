# Benchkit (`bk`)

A minimal, self-contained command benchmarking tool written in Rust.
Think of it as a simplified [`hyperfine`](https://github.com/sharkdp/hyperfine) — same idea,
built from scratch so you understand every moving part.

---

## What it does

- Runs any shell command **N times** with configurable **warmup** passes
- Reports **mean, stddev, min, max, p50, p95, p99** in a clean table
- Warns when variance is suspiciously high (rsd > 10%)
- Compares **two commands** side-by-side with a speedup ratio
- Outputs **JSON** for scripting / CI pipelines
- Shows per-run timings with `--show-runs`

---

## Installation

```bash
git clone <repo>
cd benchkit
cargo build --release
# binary is at target/release/bk
cp target/release/bk ~/.local/bin/bk
```

Requires Rust 1.70+ (uses edition 2021).

---

## Usage

### Benchmark a single command

```
bk run [OPTIONS] <CMD>

Options:
  -n, --runs <N>      Number of timed runs      [default: 10]
  -w, --warmup <N>    Warmup runs (discarded)   [default: 3]
  -l, --label <LABEL> Custom label in output
      --show-runs     Print each individual timing
      --ignore-failure Allow non-zero exit codes
      --json          Emit JSON instead of a table
```

```bash
# basic
bk run "sleep 0.1"

# custom run count and warmup
bk run -n 20 -w 5 "find /usr -name '*.so' 2>/dev/null"

# pipe-friendly JSON
bk run --json "gzip -k /tmp/bigfile" | jq '.mean'
```

### Compare two commands

```
bk cmp [OPTIONS] <CMD_A> <CMD_B>

Options:
  -n, --runs <N>         Runs per command     [default: 10]
  -w, --warmup <N>       Warmup per command   [default: 3]
      --label-a <LABEL>  Label for CMD_A
      --label-b <LABEL>  Label for CMD_B
      --ignore-failure
      --json
```

```bash
# compare two grep implementations
bk cmp "grep -r TODO ." "rg TODO ."

# with readable labels
bk cmp \
  --label-a "grep" \
  --label-b "ripgrep" \
  "grep -r TODO ." \
  "rg TODO ."

# JSON output for the diff
bk cmp --json "cat /etc/hosts" "bat /etc/hosts"
```

---

## Example output

```
  command : sleep 0.1
  runs    : 10  (+ 3 warmup)

  sleep 0.1
  ──────────────────────────────────────────────────────────────────
  mean            100.87 ms    ± 0.42 ms
  min             100.31 ms
  max             101.73 ms
  p50             100.82 ms
  p95             101.58 ms
  p99             101.71 ms
  ──────────────────────────────────────────────────────────────────
  runs                    10
  rsd                   0.4%
```

Comparison:

```
  Benchmark comparison
  ────────────────────────────────────────────────────────────────────────────────
               grep (baseline)   ripgrep (challenger)       diff         ratio
  ────────────────────────────────────────────────────────────────────────────────
  mean                 412.3 ms              89.1 ms    -323.2 ms       0.216×
  stddev                12.4 ms               3.2 ms      -9.2 ms       0.258×
  min                  399.1 ms              85.4 ms    -313.7 ms       0.214×
  max                  441.0 ms              97.3 ms    -343.7 ms       0.221×
  p50                  410.7 ms              88.8 ms    -321.9 ms       0.216×
  p95                  438.6 ms              95.7 ms    -342.9 ms       0.218×
  p99                  440.8 ms              97.1 ms    -343.7 ms       0.220×
  ────────────────────────────────────────────────────────────────────────────────
  verdict : ripgrep is 4.63× faster on average
  runs    : 10  each
```

---

## How the statistics are computed

| Stat   | Method |
|--------|--------|
| mean   | arithmetic mean of all timing samples |
| stddev | sample std dev (Bessel-corrected, N−1 denominator) |
| pN     | linear interpolation, same as `numpy.percentile` default |
| rsd    | relative std dev = stddev / mean × 100 |
| speedup ratio | `mean(A) / mean(B)` — >1 means B is faster |

Warmup runs are executed but their timings are **never** included in the
statistics. The warmup phase exists to let the OS page-in the binary,
fill disk caches, and reach a steady CPU frequency before we start
recording — exactly what `hyperfine --warmup` does.

---

## Why wall-clock time?

`bk` measures wall-clock elapsed time via `std::time::Instant`, not CPU
time. This is intentional: for most CLI tool benchmarks you care about
end-to-end latency as a user, not just CPU cycles. If you need CPU time
you can wrap `time -p <cmd>` and parse the output.

Stdout and stderr of the benchmarked command are suppressed (`/dev/null`)
so they don't skew timings with terminal I/O.

---

## Project layout

```
benchkit/
├── Cargo.toml
└── src/
    ├── main.rs       CLI entry point, subcommand dispatch
    ├── runner.rs     process execution + timed loop
    ├── stats.rs      Samples, Summary, all math
    ├── compare.rs    two-command comparison logic
    └── output.rs     table rendering + JSON serialisation
```

---

## Running the tests

```bash
cargo test
```

The unit tests in `stats.rs` cover mean, stddev, min/max, and percentile
interpolation against known values.

---

## Limitations / known issues

- Measures wall-clock time only; no CPU-time or context-switch counters.
- No shell command caching / parameter scanning (hyperfine's `{var}` syntax).
- Windows untested (uses `$SHELL` env var, falls back to `/bin/sh`).

Pull requests welcome.

---

## License

MIT