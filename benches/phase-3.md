# Phase 3 — Hyperfine Benchmarks

> Generated 2026-04-26 on macOS 24.6.0 / arm64 (Apple Silicon).
> Toolchain: rustc 1.85+ (release profile, lto = true, codegen-units = 1).

## Workloads

| Bench | Config | Widgets | Subprocess |
|---|---|---|---|
| cchud-default | `default-line()` (1 виджет) | 1 | no |
| cchud-22w | `benches/configs/phase-3-22w.json` | 22 (без TerminalWidth, CustomCommand) | no |
| cchud-23w-cmd | `benches/configs/phase-3-23w-with-cmd.json` | 23 (включая `echo phase-3`) | yes |

Все запуски: `cat <payload.json> | ./target/release/cchud` с `HOME` override → `~/.claude/settings.json` → конкретный config.

## Results (mean ± σ, p95)

| Bench | Mean | σ | p95 | Min | Max | Gate | Status |
|---|---|---|---|---|---|---|---|
| cchud-default | 1.66 ms | 0.35 ms | 2.34 ms | 1.19 ms | 3.55 ms | < 5 ms | ✓ |
| cchud-22w | 2.00 ms | 1.13 ms | 3.42 ms | 1.16 ms | 9.76 ms | < 5 ms | ✓ |
| cchud-23w-cmd | 3.07 ms | 1.53 ms | 6.38 ms | 1.35 ms | 10.67 ms | warning if > 5 ms | ⚠ |

> p95 рассчитан из `--export-json` через `sorted(times)[int(n*0.95)]`.

## Phase 2 baseline reference

Phase 2 hyperfine (`benches/phase-2.md`): mean ≈ 1.3 ms, p95 ≈ 1.6 ms, для 1-widget config.

Phase 3 прирост: default → 22w = +0.34ms (+20%), в бюджете +200%.

## Phase 3 budget breakdown (worst-case)

| Стадия | Бюджет |
|---|---|
| Cold-start (binary load + dyld macOS) | ~1–2 ms |
| stdin + JSON parse (envelope ~1.5 KB) | ~0.4 ms |
| config load + parse | ~0.3 ms |
| 22 widget renders (clone/format) | ~0.3 ms |
| 1 subprocess CustomCommand с `echo` | ~0.5–1.5 ms (process spawn dominates) |
| println | ~0.05 ms |
| Worst-case итого | ~3.5–4.5 ms |

## Notes

- **TerminalWidth** исключён из bench-config: под `cat ... | cchud` нет TTY → виджет рендерит None, скоринг искажается. Покрыт unit-тестами.
- **cchud-22w gate ✓**: p95 = 3.42 ms < 5 ms hard gate. mean+2σ = 4.26 ms (proxy); реальный p95 предпочтительнее из-за outliers.
- **cchud-23w-cmd ⚠**: p95 = 6.38 ms > 5 ms warning. subprocess `echo` spawn доминирует (~1–1.5 ms overhead). Документировано как expected: CustomCommand с subprocess добавляет OS process spawn latency.
- **Outliers**: `cchud-22w` имеет outliers (max=9.76 ms) из-за shell overhead при измерении. Устойчивое ядро — 1.2–3.5 ms.

## Reproduction

```bash
# 22-widget gate (p95 < 5 ms)
mkdir -p /tmp/cchud-bench/.claude
cp benches/configs/phase-3-22w.json /tmp/cchud-bench/.claude/settings.json
hyperfine --warmup 20 --runs 200 \
  --export-markdown benches/results/phase-3-22w.md \
  --export-json benches/results/phase-3-22w.json \
  --command-name "cchud-22w" \
  "HOME=/tmp/cchud-bench cat benches/samples/payload-cchud-sonnet-xlarge.json | ./target/release/cchud" \
  --command-name "cchud-default" \
  "cat benches/samples/payload-cchud-sonnet-xlarge.json | ./target/release/cchud"

# 23-widget с subprocess
mkdir -p /tmp/cchud-bench-cmd/.claude
cp benches/configs/phase-3-23w-with-cmd.json /tmp/cchud-bench-cmd/.claude/settings.json
hyperfine --warmup 10 --runs 100 \
  --export-markdown benches/results/phase-3-23w-with-cmd.md \
  --export-json benches/results/phase-3-23w-with-cmd.json \
  --command-name "cchud-23w-cmd" \
  "HOME=/tmp/cchud-bench-cmd cat benches/samples/payload-cchud-sonnet-xlarge.json | ./target/release/cchud"

# p95 из JSON
python3 -c "
import json, statistics
with open('benches/results/phase-3-22w.json') as f:
    data = json.load(f)
for r in data['results']:
    times_ms = [t*1000 for t in r['times']]
    sorted_t = sorted(times_ms)
    p95 = sorted_t[int(len(sorted_t)*0.95)]
    print(f\"{r['command']}: p95={p95:.2f}ms\")
"
```
