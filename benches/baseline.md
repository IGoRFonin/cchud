| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `cat payload-sonnet-medium.json \| npx -y ccstatusline@latest` | 827.7 ± 28.4 | 792.5 | 933.0 | 3.35 ± 0.18 |
| `cat payload-sonnet-medium.json \| ccstatusline` | 246.7 ± 10.0 | 235.3 | 321.0 | 1.00 |

## Контекст замера

- **Машина:** Apple M4 Pro / 24 GB
- **OS:** macOS 15.6.1
- **node:** v24.13.0
- **ccstatusline:** 2.2.8
- **hyperfine:** 1.20.0
- **Sample:** `payload-sonnet-medium.json`, 1081 байт, сценарий: git-clean medium transcript
- **Дата:** 2026-04-26T05:45:34Z

## RAM (пик RSS, ccstatusline global)

- maximum resident set size: 102 465 536 байт ≈ 97.7 МБ
- peak memory footprint: 44 259 616 байт ≈ 42.2 МБ

## Цели cchud (из PRD §2)

- cold-start p95 (M-серия): **< 5 мс**  ← запас от текущего baseline (global 246.7 мс): ×49
- RSS пик: **< 5 МБ**  ← запас от текущего baseline (97.7 МБ): ×20

## Замечание о результатах

PRD §1 ожидал npx ~50–150 мс, global — «заметно быстрее». Фактически:
- npx: **827.7 мс** (в 5–16× медленнее ожидания)
- global: **246.7 мс** (в 5–50× медленнее ожидания)

Возможная причина: node.js cold-start на macOS с nvm занимает ~100–200 мс само по себе;
ccstatusline читает и обрабатывает transcript-файл через JSONL-кэш при каждом вызове.
Зафиксировано в DECISIONS.md как pre-flight delta для пересмотра PRD §1 в фазе 1.
