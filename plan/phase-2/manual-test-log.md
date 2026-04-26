# Phase 2 — Manual real-CC test log

> **Status:** PENDING — заполнить после ручного теста в реальном Claude Code.

**Date:** YYYY-MM-DD
**Tester:** Igor Fonin
**Chip:** Apple M4 Pro
**OS:** Darwin 24.6.0 (macOS)
**Claude Code version:** (см. `cat ~/.claude/version` или Help → About)
**cchud version:** 0.0.1

## Setup
- [ ] Backup `~/.claude/settings.json` → `.pre-phase2-backup`
- [ ] `./target/release/cchud install` (или `--force`)
- [ ] Открыт Claude Code

## Test session
- **Длительность:** XX минут
- **Действий:** N сообщений, M tool-calls
- **Модели:** (если переключал — список)

## Observations
| # | Что | Результат |
|---|---|---|
| 1 | StatusLine отображает display_name модели | ✓ / ✗ |
| 2 | StatusLine обновляется при смене модели | ✓ / ✗ / N/A |
| 3 | Нет видимых лагов в UI Claude Code | ✓ / ✗ |
| 4 | Нет cchud-related записей в Claude Code диагностике / Console.app | ✓ / ✗ |
| 5 | settings.json после теста — diff показывает только statusLine | ✓ / ✗ |

## Notes / issues
(Любые наблюдения, неожиданности, idea'и для будущих фаз.)

## Sign-off
- [ ] Все 5 observations ✓
- [ ] Готов проставить Phase 2 как `[x]` в `plan/README.md`
