# Phase 3 — Manual Real-CC Test Log

> Шаблон под Task 9. Заполняется при ручной верификации в реальном Claude Code.

## Тестовая сессия

- **Дата:** TBD (проставить при выполнении)
- **Версия cchud:** 0.1.0-alpha (`cchud --version`)
- **Версия Claude Code:** TBD (`claude --version`)
- **OS / arch:** macOS 24.x / arm64 (M-серия)

## Сценарий

1. `cchud install` — wires в `~/.claude/settings.json`.
2. `~/.config/cchud/settings.json` — конфиг с включёнными:
   - `model`
   - `worktree-name` + `worktree-branch` + `worktree-mode`
   - `vim-mode`
   - `session-cost` + `session-clock`
   - `context-percentage` + `context-bar` (width 10)
   - `version` + `claude-session-id`
   - `custom-text` "demo" + `custom-command` `echo phase-3`
3. Открыть Claude Code в worktree (`git worktree add ../cchud-wt feature/demo`).
4. Включить vim mode.
5. Поработать ≥ 5 минут (любые задачи: чтение/правки файлов, run tests).

## Наблюдения

- [ ] Statusline рендерится без артефактов
- [ ] Worktree-name и Worktree-branch показывают правильные значения
- [ ] Vim mode переключается NORMAL ⟷ INSERT при `i` / `Esc`
- [ ] SessionCost растёт по мере работы
- [ ] SessionClock тикает (HH:MM:SS / MM:SS форматы)
- [ ] ContextPercentage обновляется
- [ ] ContextBar заполняется пропорционально
- [ ] CustomCommand "echo phase-3" печатает "phase-3"
- [ ] Лагов на UI Claude Code нет
- [ ] Stderr CC чист от `cchud:` warning'ов

## Метрики

- p95 cchud render time (из `hyperfine` Task 8): TBD ms
- Binary size (`ls -lh target/release/cchud`): TBD MB

## Замечания / отклонения от ожиданий

(Сюда — любые сюрпризы: payload поля в реальном CC отличаются от наших synthetic, отсутствуют ожидаемые виджеты, и т.п. Дата + описание.)

## Sign-off

- [ ] Все 10 чекбоксов выше отмечены
- [ ] manual-test-log.md committed в `plan/phase-3/`
