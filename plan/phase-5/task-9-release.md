# Task 9 — Release 0.3.0

**Files:**
- Modify: `Cargo.toml` (`version = "0.3.0"`)
- Modify: `Cargo.lock` (auto)
- Modify: `CHANGELOG.md` (новая секция `## 0.3.0 — Git widgets`)
- Modify: `README.md` (widget table 26→46/60, deps, quickstart с git-конфигом)
- Modify: `docs/widgets.md` (20 git-виджетов TODO → DONE)
- Modify: `plan/README.md` (Phase 5 → `[x]`, Phase 6 → `[~]`)
- Create: `plan/phase-5/manual-test-log.md` (шаблон + результат запуска)
- Tag: `v0.3.0` (push после merge в main)

## Goal

Финализация Phase 5 — bump версии, документация, tag. Никаких code-изменений в `src/` (вся работа сделана в T1–T8).

## Inputs

- T8 закрыт: snapshot'ы committed, hyperfine `p95 < 8 ms` зафиксирован.
- `cargo test --locked` зелёный, CI matrix зелёный 2 раза подряд.
- Manual test проведён на этом репо с активным `GitPr`.

---

- [ ] **Step 1: Bump version в `Cargo.toml`**

Edit `Cargo.toml`:
- `old_string`: `version = "0.2.0"`
- `new_string`: `version = "0.3.0"`

```bash
cargo build --release --locked
target/release/cchud --version
```

Expected: `cchud 0.3.0`.

- [ ] **Step 2: Обновить `CHANGELOG.md`**

Read текущий `CHANGELOG.md`. Найти заголовок `# Changelog` (или аналог). Добавить новую секцию **выше** записи `0.2.0`:

```markdown
## 0.3.0 — 2026-04-XX

### Added (Phase 5 — Git widgets)

20 git-виджетов через `gix 0.81` (pure Rust, без libgit2):

**Head (3):** `git-branch`, `git-sha` (7-char short), `git-root-dir`

**Status (6):** `git-status` (summary `M1 ~2 ?3 ✗4`), `git-changes`, `git-staged`, `git-unstaged`, `git-untracked`, `git-conflicts`

**Diff stat (2):** `git-insertions` (`+N`), `git-deletions` (`-N`) — lazy, считаются только при наличии в строке.

**Tracking (1):** `git-ahead-behind` (`↑3↓1`)

**Remote (7):** `git-origin-{owner,repo,owner-repo}`, `git-upstream-{owner,repo,owner-repo}`, `git-is-fork`. Hand-parser URL без regex (4 формата: SSH short/explicit, HTTPS с/без `.git`).

**HTTP (1):** `git-pr` через GitHub API. Auth priority: `GITHUB_TOKEN` → `gh auth token` → анонимно. Дисковый кэш `~/.cache/cchud/pr-cache.bincode` с TTL 30 s, hard timeout 200 ms, offline soft-fail.

### Performance

- p95 < 8 ms на 21-widget config (включая `git-pr` cache-hit) — см. `benches/phase-5.md`
- `git-pr` cache-hit overhead: ~0.4 ms vs baseline без HTTP виджета
- Один `gix::status` вызов на 6 status-виджетов через `OnceCell`

### Internals

- `src/git/mod.rs`: `GitInfo` lazy через `OnceCell` для `status_counts`/`diff_stat`/`tracking`
- `RenderContext::git()` — единственная точка входа, `discover()` максимум один раз за render
- `src/git/remote.rs`: hand-parser, экономит ~300 KB бинаря vs `regex` dep
- `src/git/pr.rs`: schema-versioned `PrCache` (mismatch → silent reset)

### Dependencies

- `gix = "=0.81.0"` (pinned exact, runtime)
- `ureq = "2"` with `rustls-tls` (runtime — для GitPr)
- `bincode = "1"` (runtime — кэш)
- `mockito = "1"` (dev-dep, GitHub API mock)

### Decisions

См. `docs/DECISIONS.md` D-2026-04-27 — gix vs git2.

### Scope notes

5 worktree-виджетов (`worktree`, `worktree-mode`, `worktree-name`, `worktree-branch`, `worktree-original-branch`) уже были реализованы в Phase 3 через `payload.worktree`. Phase 5 их не дублирует. Total widget coverage: 46/60.
```

`file_path`: `/Users/igor/mp/startup/cchud/CHANGELOG.md`.

- [ ] **Step 3: Обновить `README.md`**

Read `README.md`. Найти секцию "supported widgets" (или похожую таблицу из Phase 3 / Phase 4 release).

Заменить статус 20 git-виджетов с `❌`/`TODO` на `✅`/`DONE`. Cumulative: 46/60.

Если в README есть quickstart — добавить пример конфига с git:

```json
{
  "lines": [{
    "widgets": [
      { "type": "model" },
      { "type": "separator" },
      { "type": "git-branch" },
      { "type": "git-status" },
      { "type": "separator" },
      { "type": "context-percentage" },
      { "type": "session-cost" }
    ]
  }]
}
```

Если есть deps-таблица — добавить gix/ureq/bincode.

В секции "Performance" обновить цифру: `< 8 ms p95 на 21 виджет (включая git+PR cache-hit)`.

В секции "Roadmap" / "What's next" — Phase 6 (transcript widgets) теперь next.

- [ ] **Step 4: Обновить `docs/widgets.md`**

Read `docs/widgets.md`. Найти 20 строк с `TODO` в Phase 5.

Заменить статус каждой строки `TODO` → `DONE`:

```markdown
| `git-branch` | 5 | git | mid | DONE | gix: HEAD branch name |
| `git-changes` | 5 | git | mid | DONE | gix: changed file count |
...etc
```

Обновить секцию "Сводка по фазам":
```markdown
| 5 | 20 (git + git-pr) | 46 |
```
(статус цифры могут уже быть верными, проверить).

- [ ] **Step 5: Обновить `plan/README.md`**

Edit `plan/README.md`:
- `old_string`:
  ```
  - [~] Фаза 4 — Powerline (next)
  - [ ] Фаза 5 — Git widgets
  ```
  (или текущее состояние Phase 4/5 — поправить точечно)
- `new_string`:
  ```
  - [x] Фаза 4 — Powerline
  - [x] Фаза 5 — Git widgets
  - [~] Фаза 6 — Transcript (next)
  ```

(Если Phase 4 уже `[x]`, оставить и менять только Phase 5/6.)

В таблице релизов проверить — `0.3.0` в строке Phase 5 уже стоит, ничего менять не нужно.

- [ ] **Step 6: Создать `plan/phase-5/manual-test-log.md`**

Шаблон:

```markdown
# Phase 5 — Manual real-CC test log

**Date:** 2026-04-XX
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

1. [ ] Открыть Claude Code в этом репо
2. [ ] Подождать первого рендера statusline (cache-miss → ~200ms)
3. [ ] Проверить, что `git-branch = main`, `git-status` пусто (clean)
4. [ ] Сделать `touch test.txt`, перерендерить — `git-status = ?1`, `git-changes = 1`
5. [ ] Удалить файл, рендер вернулся к None
6. [ ] Проверить `~/.cache/cchud/pr-cache.bincode` — файл есть
7. [ ] Если на ветке есть открытый PR — `git-pr = "PR #N"`
8. [ ] Запустить ≥ 5 минут активной сессии — нет регрессий, statusline стабилен

## Observations

- Cache-hit рендер: < 5 ms (визуально мгновенно)
- Cache-miss первый: заметная пауза ~200 ms (ОК, разовая)
- При offline (Wi-Fi выкл) виджет git-pr молча возвращает None — pipeline не ломается ✅

## Issues found

(нет / список)

## Conclusion

✅ Phase 5 готова к релизу.
```

Path: `/Users/igor/mp/startup/cchud/plan/phase-5/manual-test-log.md`.

Заполнить реальными результатами после прохождения теста.

- [ ] **Step 7: Финальная проверка standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
target/release/cchud --version
```

Expected:
- Все exit 0
- `cchud 0.3.0`

- [ ] **Step 8: Run manual test (Step 6 чек-лист)**

Заполнить `plan/phase-5/manual-test-log.md` реальными observations.

- [ ] **Step 9: Commit release-changes**

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md README.md docs/widgets.md plan/README.md plan/phase-5/manual-test-log.md
git commit -m "release: 0.3.0 — git widgets

Phase 5 complete. Cumulative widget coverage: 46/60.

20 new git widgets via gix 0.81 (pure Rust):
- 3 head (branch, sha, root-dir)
- 6 status (status summary + 5 counts)
- 2 diff stat (insertions, deletions, lazy)
- 1 tracking (ahead-behind)
- 7 remote (origin/upstream owner/repo + is-fork)
- 1 PR via GitHub API (cached, offline-tolerant)

Performance: p95 < 8 ms on 21-widget config including git-pr cache-hit.

See CHANGELOG.md for details, benches/phase-5.md for hyperfine results,
docs/DECISIONS.md D-2026-04-27 for gix vs git2 rationale.
"
```

- [ ] **Step 10: Tag and push**

```bash
git tag -a v0.3.0 -m "Phase 5 — Git widgets (20 widgets, 46/60 cumulative)"
git push origin main
git push origin v0.3.0
```

Verify on GitHub:
```bash
gh release view v0.3.0 2>/dev/null || echo "tag pushed, no release yet"
gh api /repos/IGoRFonin/cchud/git/refs/tags/v0.3.0 -q .ref
```

Expected: `refs/tags/v0.3.0` существует.

- [ ] **Step 11: Создать GitHub Release (опционально)**

```bash
gh release create v0.3.0 \
  --title "v0.3.0 — Git widgets" \
  --notes "$(awk '/^## 0\.3\.0/{flag=1; next} /^## 0\.2\.0/{flag=0} flag' CHANGELOG.md)"
```

(Извлекает секцию `## 0.3.0` из CHANGELOG в release notes.)

- [ ] **Step 12: Verification**

```bash
git tag --list 'v0.3.*'
target/release/cchud --version
grep -c '## 0.3.0' CHANGELOG.md
grep -c 'Phase 5 — Git widgets' plan/README.md
ls plan/phase-5/manual-test-log.md
```

Expected:
```
v0.3.0
cchud 0.3.0
1
1
exists
```

- [ ] **Step 13: Smoke check on installed binary**

```bash
cd /tmp && rm -rf cchud-smoke && mkdir cchud-smoke && cd cchud-smoke && git init -b main -q && git -c user.email=a@b -c user.name=a commit --allow-empty -q -m init
echo '{"session_id":"x","model":{"id":"m","display_name":"M"},"workspace":{"current_dir":"'$(pwd)'"}}' \
  | /Users/igor/mp/startup/cchud/target/release/cchud
```

Expected: вывод корректен (минимум `M`, остальное зависит от default-config).

## Verification

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
target/release/cchud --version  # → 0.3.0
git tag --list 'v0.3.0'         # → v0.3.0
```

## Definition of Done

- [ ] `Cargo.toml` version = `0.3.0`
- [ ] `target/release/cchud --version` показывает `cchud 0.3.0`
- [ ] `CHANGELOG.md` содержит секцию `## 0.3.0` с полным описанием
- [ ] `README.md` обновлён: widget table 46/60, deps gix/ureq/bincode, performance цифра
- [ ] `docs/widgets.md`: 20 git-виджетов в DONE
- [ ] `plan/README.md`: Phase 5 = `[x]`, Phase 6 = `[~]`
- [ ] `plan/phase-5/manual-test-log.md` заполнен реальными observations
- [ ] Manual test пройден ≥ 5 минут на этом репо без регрессий
- [ ] Один commit `release: 0.3.0 — git widgets`
- [ ] Tag `v0.3.0` создан и запушен
- [ ] (Опционально) GitHub Release создан

## Files touched

- `Cargo.toml`, `Cargo.lock` (modified)
- `CHANGELOG.md`, `README.md` (modified)
- `docs/widgets.md` (modified)
- `plan/README.md` (modified)
- `plan/phase-5/manual-test-log.md` (created)
- Tag `v0.3.0` (push)

## Risks & rollback

- **README табличка 46/60 расходится с реальностью**: посчитать через `grep -c 'DONE' docs/widgets.md` после Step 4. Должно быть 46.
- **`gh release create` падает**: опциональный шаг — пропустить, tag достаточен.
- **Push на main без PR**: проверить, есть ли в репо требование PR-ревью. Если есть — открыть PR от ветки `phase-5/release-0.3.0` через `gh pr create`, дождаться merge.
- **Tag нельзя двинуть после push**: если найден баг после tag — fix в `0.3.1`, не трогаем `v0.3.0`. Standard semver.
- **Manual test fails**: записать в `manual-test-log.md`, открыть issue, **не релизить**. Tag создаём только после ✅.
- **Rollback**: `git tag -d v0.3.0 && git push origin :refs/tags/v0.3.0` — если успели заметить до consumer'ов. После — только `0.3.1`.
