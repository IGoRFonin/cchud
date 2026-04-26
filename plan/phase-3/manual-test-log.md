# Phase 3 — Manual Real-CC Test Log

> Шаблон под Task 9. Заполняется при ручной верификации в реальном Claude Code.

## Тестовая сессия

- **Дата:** 2026-04-27
- **Версия cchud:** 0.1.0-alpha (`cchud --version`)
- **Версия Claude Code:** 2.1.119
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

- [x] Statusline рендерится без артефактов
- [x] Worktree-name и Worktree-branch показывают правильные значения
- [x] Vim mode переключается NORMAL ⟷ INSERT при `i` / `Esc`
- [x] SessionCost растёт по мере работы
- [x] SessionClock тикает (HH:MM:SS / MM:SS форматы)
- [x] ContextPercentage обновляется
- [x] ContextBar заполняется пропорционально
- [x] CustomCommand "echo phase-3" печатает "phase-3"
- [x] Лагов на UI Claude Code нет
- [x] Stderr CC чист от `cchud:` warning'ов

## Метрики

- p95 cchud render time (из `hyperfine` Task 8): < 5 ms (gate passed в T8)
- Binary size (`ls -lh target/release/cchud`): 589 KB

## Замечания / отклонения от ожиданий

- 2026-04-27: Конфиг виджетов находится в `~/.config/cchud/settings.json` (не в блоке `cchud` внутри `~/.claude/settings.json` — документация исправлена).

## Sign-off

- [x] Все 10 чекбоксов выше отмечены
- [x] manual-test-log.md committed в `plan/phase-3/`
