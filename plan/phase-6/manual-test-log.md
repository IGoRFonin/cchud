# Phase 6 — Manual battle-test log

**Дата:** 2026-04-29
**Тестировщик:** Igor Fonin
**Версия:** cchud 0.4.0 (commit `b39706e`)
**Окружение:** mac mini M2 / iTerm2 / Claude Code 1.X
**Конфиг:** `model | git-branch | block-timer | tokens-total | input-speed | thinking-effort`

## Сценарий

1. Запущена активная CC-сессия в репо `cchud` (этот же репозиторий).
2. ~7 пар user/assistant за 5 минут с миксом `thinking.effort` (`high`, `max`).
3. После 5 минут — `rm ~/.cache/cchud/transcript-*.bincode` и проверка следующего рендера.
4. После — новая сессия (`/clear`), проверка чистого кэша.

## Результаты

- [x] `BlockTimer` корректно показывает `⏰ HH:MM:SS`, монотонно уменьшается.
- [x] `TokensTotal` инкрементируется после assistant-ответа.
- [x] `InputSpeed` обновляется (значение меняется между ответами).
- [x] `ThinkingEffort` показывает `🧠 high` / `🧠 max` после соответствующих ответов.
- [x] Cache file размер: ~45 KB на ~5 MB transcript = ~0.9% (target ≤ 5%).
- [x] Cache wipe → следующий рендер ОК (~9 ms cold; не повисает).
- [x] `/clear` → новый кэш создаётся; старые статы не leak'ают.

## Найденные баги

Баги не обнаружены. Все critical-проверки пройдены.

## Заключение

- [x] PASS — все critical-проверки прошли.
- [ ] BLOCKED — баги, нельзя tag'ать.
