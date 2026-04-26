# План: cchud — Rust-порт ccstatusline

> PRD: [`../docs/prd-cchud.md`](../docs/prd-cchud.md)
> Upstream ресерч: [`../ccstatusline-research.md`](../ccstatusline-research.md)
> Лицензия: MIT
> Автор: igorfonin

План разбит на 11 фаз. Каждая фаза — отдельный файл с целью, шагами, exit-criteria и связями. После каждой фазы — рабочий бинарь, который можно использовать.

## Фазы

| # | Файл | Длительность | Exit |
|---|---|---|---|
| 0 | [`phase-0-checks.md`](./phase-0-checks.md) | 1–2 ч | Baseline-числа, payload-семплы, проверка имён |
| 1 | [`phase-1-init.md`](./phase-1-init.md) | 1–2 ч | CI зелёный, скелет, snapshot-тесты |
| 2 | [`phase-2-pipeline.md`](./phase-2-pipeline.md) | 3–5 дн | Виджет `Model` работает в реальном Claude Code |
| 3 | [`phase-3-mvp.md`](./phase-3-mvp.md) | 1 нед | **0.1.0 alpha** — топ-10 виджетов |
| 4 | [`phase-4-powerline.md`](./phase-4-powerline.md) | 3–4 дн | Powerline визуальный паритет |
| 5 | [`phase-5-git-widgets.md`](./phase-5-git-widgets.md) | 3–5 дн | Все 20+ git-виджетов |
| 6 | [`phase-6-transcript.md`](./phase-6-transcript.md) | 4–6 дн | JSONL-кэш + cost/usage/tokens/blocks |
| 7 | [`phase-7-other-widgets.md`](./phase-7-other-widgets.md) | 2–3 дн | **0.5.0** — паритет 60+ виджетов |
| 8 | [`phase-8-tui.md`](./phase-8-tui.md) | 1–2 нед | TUI-конфигуратор UX-паритет |
| 9 | [`phase-9-distribution.md`](./phase-9-distribution.md) | 3–5 дн | **1.0.0** — npm/brew/curl |
| 10 | [`phase-10-future.md`](./phase-10-future.md) | — | Длинный хвост (1.x) |

**Суммарно:** ~6–8 недель полной занятости до 1.0.0 / 3–4 месяца на пет-проектном темпе.

## Принципы

1. **Cold-start budget:** на каждый PR — hyperfine. Регрессия > 10% от p95 блокирует merge.
2. **Snapshot-first:** реальные payload'ы из Фазы 0 — основа всех тестов.
3. **Lazy всё подряд:** git/transcript/HTTP инициализируются только если виджет реально нужен.
4. **MIT lic + явная атрибуция ccstatusline upstream** в README и LICENSE.
5. **Никаких npm-резолвов** в горячем пути. npm-пакет — только bootstrap-loader.

## Контракты между фазами

```
Фаза 0 (payload-семплы, upstream-map) ──┐
                                        ├──► Фаза 1 (skeleton + CI)
                                        │
                              Фаза 2 (pipeline + types + Model)
                                        │
                              Фаза 3 (MVP виджеты)
                                        │
            ┌───────────────────────────┴───────────────────────────┐
            ▼                                                       ▼
   Фаза 4 (Powerline)                                  Фаза 5 (git-виджеты)
            │                                                       │
            └───────────────┬───────────────────────────────────────┘
                            ▼
                  Фаза 6 (transcript + JSONL-кэш)
                            │
                            ▼
                  Фаза 7 (остальные виджеты)
                            │
                            ▼
                  Фаза 8 (TUI-конфигуратор)
                            │
                            ▼
                  Фаза 9 (release 1.0.0)
                            │
                            ▼
                  Фаза 10 (длинный хвост)
```

Фазы 4 и 5 могут идти **параллельно** после 3, если есть второй разработчик.

## Релизы

| Версия | После фазы | Содержание |
|---|---|---|
| 0.1.0-alpha | 3 | Топ-10 виджетов, plain renderer |
| 0.3.0 | 5 | + Powerline + все git-виджеты |
| 0.5.0 | 7 | + transcript-виджеты, паритет фич |
| 0.9.0 | 8 | + TUI-конфигуратор |
| 1.0.0 | 9 | + дистрибуция (npm/brew/curl), доки |

## Текущий статус

- [x] Ресерч (`ccstatusline-research.md`)
- [x] PRD (`docs/prd-cchud.md`)
- [x] План разбит по фазам
- [x] Фаза 0 — Проверки
- [~] Фаза 1 — Init (in progress)
- [ ] Фаза 2 — Pipeline
- [ ] Фаза 3 — MVP
- [ ] Фаза 4 — Powerline
- [ ] Фаза 5 — Git widgets
- [ ] Фаза 6 — Transcript
- [ ] Фаза 7 — Остальные виджеты
- [ ] Фаза 8 — TUI
- [ ] Фаза 9 — Distribution
- [ ] Фаза 10 — Future
