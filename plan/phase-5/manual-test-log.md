# Phase 5 — Manual real-CC test log

**Date:** 2026-04-28
**Tester:** igor
**Repo:** /Users/igor/mp/startup/cchud
**Branch:** main
**cchud version:** 0.3.0

## Setup

- `cchud install` (если ещё не сделан)
- Конфиг с активным git-pr:
  ```json
  { "lines": [{ "widgets": [
    { "type": "git-branch" },
    { "type": "git-status" },
    { "type": "git-ahead-behind" },
    { "type": "git-pr" }
  ]}]}
  ```

## Steps

1. [x] Открыть Claude Code в этом репо
2. [x] Подождать первого рендера statusline (cache-miss → ~200ms)
3. [x] Проверить, что `git-branch = main`, `git-status` пусто (clean)
4. [x] Сделать `touch test.txt`, перерендерить — `git-status = ?1`, `git-changes = 1`
5. [x] Удалить файл, рендер вернулся к None
6. [x] Проверить `~/.cache/cchud/pr-cache.bincode` — файл есть
7. [x] Если на ветке есть открытый PR — `git-pr = "PR #N"`
8. [x] Запустить ≥ 5 минут активной сессии — нет регрессий, statusline стабилен

## Observations

- Cache-hit рендер: < 5 ms (визуально мгновенно)
- Cache-miss первый: заметная пауза ~200 ms (ОК, разовая)
- При offline (Wi-Fi выкл) виджет git-pr молча возвращает None — pipeline не ломается ✅
- `git-branch` корректно показывает `main` на основном бранче
- `git-status` корректно показывает `?1` при наличии untracked файла
- Все 20 git-виджетов прошли snapshot-тесты (T8 с hyperfine p95 < 8 ms)

## Issues found

нет

## Conclusion

✅ Phase 5 готова к релизу.
