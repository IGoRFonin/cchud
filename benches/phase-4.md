# Phase 4 — Hyperfine Benchmarks (Powerline)

> Generated 2026-04-27 on macOS 24.6.0 / arm64 (Apple Silicon).
> Toolchain: rustc 1.85+ (release profile, lto = true, codegen-units = 1).

## Workloads

| Bench | Config | Widgets | Theme |
|---|---|---|---|
| phase-3-plain | `benches/configs/phase-3-23w-with-cmd.json` | 23 (включая `echo phase-3`) | plain (default) |
| phase-4-powerline | `benches/configs/phase-4-23w-powerline.json` | 23 (включая `echo phase-4`) | powerline / dracula |

Все запуски: `cat <payload.json> | CCHUD_CONFIG=<cfg> target/release/cchud` с `CCHUD_TEST_COLOR_LEVEL=true-color`.

## Results (mean ± σ, p95)

| Bench | Mean | σ | p95 | Min | Max | Gate | Status |
|---|---|---|---|---|---|---|---|
| phase-3-plain | 2.56 ms | 0.75 ms | 3.97 ms | 1.38 ms | 6.10 ms | p95 ≤ Phase 3 + 10% (7.02 ms) | ✓ |
| phase-4-powerline | 2.30 ms | 0.68 ms | 3.41 ms | 1.07 ms | 4.73 ms | p95 ≤ 5 ms | ✓ |

> p95 рассчитан из `--export-json` через `sorted(times)[int(n*0.95)]`.

**Вывод:** регрессии нет. Powerline-рендер (+ANSI-escape сегментация, UTF-8 разделители) добавляет ~0 мс к p95 относительно plain. Оба бенча в бюджете.

## Phase 3 baseline reference

Phase 3 (cchud-23w-cmd): mean ≈ 3.07 ms, p95 ≈ 6.38 ms.

Phase 4 Powerline vs Phase 3 plain: p95 3.41 ms vs 6.38 ms — на 46% ниже (faster hardware conditions + `--setup` env injection vs `HOME` override).

## Notes

- Измерения проведены с `--setup 'export CCHUD_TEST_COLOR_LEVEL=true-color'` — env var инжектируется через shell setup, не через `HOME` override.
- **Outliers**: hyperfine предупреждает о shell calibration при p < 5 ms. Стабильное ядро — 1–4 ms.
- **powerline gate ✓**: p95 = 3.41 ms < 5 ms hard gate.
- **plain gate ✓**: p95 = 3.97 ms < Phase 3 + 10% (7.02 ms).

## Reproduction

```bash
hyperfine --warmup 5 --runs 100 \
  --export-json benches/results/phase-4-final.json \
  --setup 'export CCHUD_TEST_COLOR_LEVEL=true-color' \
  --command-name "phase-3-plain" \
  "cat benches/samples/payload-cchud-sonnet-xlarge.json | CCHUD_CONFIG=benches/configs/phase-3-23w-with-cmd.json target/release/cchud" \
  --command-name "phase-4-powerline" \
  "cat benches/samples/payload-cchud-sonnet-xlarge.json | CCHUD_CONFIG=benches/configs/phase-4-23w-powerline.json target/release/cchud"

# p95 из JSON
python3 -c "
import json, statistics
with open('benches/results/phase-4-final.json') as f:
    data = json.load(f)
for r in data['results']:
    times_ms = [t*1000 for t in r['times']]
    sorted_t = sorted(times_ms)
    p95 = sorted_t[int(len(sorted_t)*0.95)]
    mean = statistics.mean(times_ms)
    stdev = statistics.stdev(times_ms)
    print(f\"{r['command']}: mean={mean:.2f}ms σ={stdev:.2f}ms p95={p95:.2f}ms\")
"
```
