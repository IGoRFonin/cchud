# Phase 2 — Hyperfine baseline

**Date:** 2026-04-26
**Chip:** Apple M4 Pro
**OS:** Darwin 24.6.0
**Rust:** rustc 1.87.0 (17067e9ac 2025-05-09) (Homebrew)
**cchud version:** 0.0.1
**ccstatusline version:** 2.2.8 (npm global)

### payload-cchud-opus-xlarge.json

Benchmark 1: ./target/release/cchud
  Time (mean ± σ):       1.4 ms ±   0.3 ms    [User: 0.6 ms, System: 0.5 ms]
  Range (min … max):     1.1 ms …   2.5 ms    200 runs
 
  Warning: Statistical outliers were detected. Consider re-running this benchmark on a quiet system without any interferences from other programs. It might help to use the '--warmup' or '--prepare' options.
 

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `./target/release/cchud` | 1.4 ± 0.3 | 1.1 | 2.5 | 1.00 |


### payload-cchud-sonnet-xlarge.json

Benchmark 1: ./target/release/cchud
  Time (mean ± σ):       1.8 ms ±   0.6 ms    [User: 0.7 ms, System: 0.7 ms]
  Range (min … max):     1.3 ms …   6.1 ms    200 runs
 
  Warning: Statistical outliers were detected. Consider re-running this benchmark on a quiet system without any interferences from other programs. It might help to use the '--warmup' or '--prepare' options.
 

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `./target/release/cchud` | 1.8 ± 0.6 | 1.3 | 6.1 | 1.00 |


### payload-cchud-sonnet-xlarge2.json

Benchmark 1: ./target/release/cchud
  Time (mean ± σ):       1.7 ms ±   1.8 ms    [User: 0.6 ms, System: 0.5 ms]
  Range (min … max):     1.2 ms …  18.8 ms    200 runs
 
  Warning: Statistical outliers were detected. Consider re-running this benchmark on a quiet system without any interferences from other programs. It might help to use the '--warmup' or '--prepare' options.
 

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `./target/release/cchud` | 1.7 ± 1.8 | 1.2 | 18.8 | 1.00 |


### Phase 2 comparison: cchud vs ccstatusline

Sample: payload-cchud-sonnet-xlarge.json

Benchmark 1: cchud
  Time (mean ± σ):       1.3 ms ±   0.2 ms    [User: 0.5 ms, System: 0.4 ms]
  Range (min … max):     1.1 ms …   2.3 ms    200 runs
 
  Warning: Statistical outliers were detected. Consider re-running this benchmark on a quiet system without any interferences from other programs. It might help to use the '--warmup' or '--prepare' options.
 
Benchmark 2: ccstatusline
  Time (mean ± σ):     230.3 ms ±   9.4 ms    [User: 140.7 ms, System: 74.4 ms]
  Range (min … max):   219.5 ms … 271.7 ms    200 runs
 
Summary
  cchud ran
  177.36 ± 25.61 times faster than ccstatusline

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `cchud` | 1.3 ± 0.2 | 1.1 | 2.3 | 1.00 |
| `ccstatusline` | 230.3 ± 9.4 | 219.5 | 271.7 | 177.36 ± 25.61 |


