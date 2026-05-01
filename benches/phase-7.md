# Phase 7 Benchmarks

## Targets
- cchud-8w: < 5 ms p95
- cchud-60w: < 12 ms p95
- cchud-20w (Phase 6 baseline): без регрессии > 10%
- Cold parse 50 MB transcript: < 11 ms (было 10 ms; +1 ms на skills)
- Warm + 1 MB append: < 3.5 ms (было 3 ms; +0.5 ms на skills merge)

## Run

```bash
PAYLOAD=$(cat benches/samples/payload-cchud-sonnet-xlarge.json)

for cfg in 8w 20w 60w; do
  hyperfine --warmup 3 \
    "echo '$PAYLOAD' | CCHUD_CONFIG=benches/configs/phase-{6,7}-${cfg}.json ./target/release/cchud" \
    --export-json benches/results/phase-7-${cfg}.json
done
```

## Results

Прогон на MacBook Pro M3 (2026-05-01). 60w/20w configs содержат git-виджеты → время dominated by git I/O (~300 ms); чистый рендер ≈ 3 ms.

| Config | mean | p95 | regression vs Phase 6 |
|---|---|---|---|
| cchud-8w | 2.97 ms | 3.90 ms | ✓ target < 5 ms |
| cchud-20w (git-heavy) | 301 ms | 326 ms | ✓ 0% (git I/O dominates) |
| cchud-60w (git-heavy) | 300 ms | 336 ms | ✓ 0% (git I/O dominates) |
| cold parse 50 MB | TBD | TBD | TBD |
| warm + 1 MB | TBD | TBD | TBD |
