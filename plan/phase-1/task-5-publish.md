# Task 5 — Publish (private GitHub repo + monitor CI)

**Files:**
- Modify: `docs/DECISIONS.md` (append Phase 1 decision), `plan/README.md` (mark Phase 1 [x])

## Goal

Создать **private** репозиторий `IGoRFonin/cchud` на GitHub, запушить все коммиты Tasks 1–4, дождаться зелёного CI на трёх OS. Переключение в public — ручное, после твоего ревью контента.

## Inputs

- Tasks 1–4 завершены: `git log --oneline | head -5` показывает 4 коммита (`chore`, `feat`, `ci`, `docs`).
- `gh` CLI установлен и залогинен (Phase 0 DECISIONS).
- Локальный `cargo build/test/clippy/fmt` зелёный.

---

- [ ] **Step 1: Pre-flight — `gh auth`**

```bash
gh auth status
```

Expected:
```
github.com
  ✓ Logged in to github.com account IGoRFonin (...)
  - Active account: true
  - Git operations protocol: ssh (или https)
  - Token scopes: 'repo', 'workflow', ...
```

Если не залогинен — `gh auth login` (interactive). Если активный аккаунт не `IGoRFonin` — `gh auth switch --user IGoRFonin`. Token scopes должны включать `repo` и `workflow`.

- [ ] **Step 2: Pre-flight — git state clean**

```bash
git status
git log --oneline | head -5
```

Expected:
- `git status`: `nothing to commit, working tree clean`.
- `git log`: 4 коммита от Tasks 1–4 в обратном порядке (`docs`, `ci`, `feat`, `chore`).

Если working tree не clean — закоммитить остатки или очистить (`git stash`).

- [ ] **Step 3: Pre-flight — final cargo gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: всё зелёное. Это последняя локальная страховка перед push.

- [ ] **Step 4: Дополнить `DECISIONS.md` записью о Phase 1 publish-стратегии**

Добавить в конец `docs/DECISIONS.md`:

```markdown
---

## 2026-04-26 — Repo visibility (Phase 1 / Task 5)

**Контекст:** Task 5 публикует skeleton в `IGoRFonin/cchud` на GitHub. PRD не требует public-видимости с момента 0.0.1; имя свободно (Phase 0 OQ-6).

**Решение:** создать репо как **private**. Переключение в public — ручное, после ревью контента (README, ATTRIBUTION, CHANGELOG, lockfile, отсутствие секретов в истории).

**Обоснование:**
- Skeleton не имеет user-facing полезности (binary печатает заглушку); public не несёт ценности до Phase 3 (0.1.0-alpha).
- Снижает риск раннего "discovery" с устаревшим README.
- CI отрабатывает одинаково в private (matrix workers + private actions), бесплатные минуты на personal account достаточны для Phase 1–7.
- При переходе в public — `gh repo edit IGoRFonin/cchud --visibility public --accept-visibility-change-consequences`.

**Последствия:** до Phase 9 распространение через `npx`/`brew` не работает (publish невозможен из private). Phase 9 переключает visibility в первой задаче дистрибуции.
```

- [ ] **Step 5: Закоммитить DECISIONS update**

```bash
git add docs/DECISIONS.md
git commit -m "docs(phase-1): record private-repo decision in DECISIONS

Phase 1 publishes to private GitHub repo. Migration to public is a
manual step after content review (intended for Phase 9 distribution).
"
```

- [ ] **Step 6: STOP-AND-CONFIRM — показать команду `gh repo create` пользователю**

**Это критическая точка плана.** Команда создаёт публичный артефакт на GitHub под аккаунтом пользователя. Не выполнять без явного "да" от пользователя.

Команда для подтверждения:

```bash
gh repo create IGoRFonin/cchud \
  --private \
  --source=. \
  --push \
  --description "Fast Rust statusline for Claude Code CLI" \
  --homepage "https://github.com/IGoRFonin/cchud"
```

**Effect:**
- Создаёт репо `github.com/IGoRFonin/cchud` (private).
- Сетит origin remote на этот репо.
- Пушит текущую ветку (main) с 5 коммитами.
- Имя `cchud` под аккаунтом `IGoRFonin` будет занято.

**Ask user:** «Готов выполнить эту команду — создаст private repo `IGoRFonin/cchud` и запушит 5 коммитов. Подтверди или дай корректировки.»

**Ждать явного подтверждения.** Если пользователь говорит "стоп", "подожди", "не сейчас" — план паузится, репо не создаётся.

- [ ] **Step 7: После подтверждения — выполнить `gh repo create`**

```bash
gh repo create IGoRFonin/cchud \
  --private \
  --source=. \
  --push \
  --description "Fast Rust statusline for Claude Code CLI" \
  --homepage "https://github.com/IGoRFonin/cchud"
```

Expected output:
```
✓ Created repository IGoRFonin/cchud on GitHub
✓ Added remote ...
✓ Pushed commits to ...
```

Если падает с "already exists" — репо уже есть. Проверить `gh repo view IGoRFonin/cchud --json visibility`. Если он уже private и пуст — `git push -u origin main`.

Если падает с "name not available" — кто-то занял имя между Phase 0 и сейчас. Открыть DECISIONS, выбрать fallback (например `cchud-rs`), обновить `Cargo.toml` `repository`, README ссылки, ATTRIBUTION ссылки, повторить.

- [ ] **Step 8: Проверить что репо создан**

```bash
gh repo view IGoRFonin/cchud --json url,visibility,name,defaultBranchRef
```

Expected:
```json
{
  "url": "https://github.com/IGoRFonin/cchud",
  "visibility": "PRIVATE",
  "name": "cchud",
  "defaultBranchRef": {"name": "main"}
}
```

Если `defaultBranchRef.name` = `master` (зависит от глобального git config) — ок, не критично; CI workflow `on: push: branches: [main]` нужно поправить или переименовать ветку: `git branch -m master main && git push -u origin main && git push origin --delete master`.

- [ ] **Step 9: Дождаться запуска и завершения CI**

```bash
# Запустить мониторинг (стримит до завершения)
gh run watch --exit-status
```

Альтернатива (polling):
```bash
gh run list --workflow=CI --limit 5 --json databaseId,status,conclusion,headBranch,name,workflowName
```

Expected (после завершения): `actionlint` job + 3 matrix-job (`test (ubuntu-latest)`, `test (macos-latest)`, `test (windows-latest)`) — все 4 со статусом `completed` и conclusion `success`.

Время прогона: `actionlint` ~30 сек, matrix ~3–7 минут (cargo build + clippy + fmt + test на каждой OS, с rust-cache cold-start первая прогонка медленнее, кэш тёплый со второго run'а).

Если matrix падает на конкретной OS:
- **ubuntu-latest:** редко падает на skeleton; проверить лог через `gh run view <id> --log-failed`.
- **macos-latest:** обычно ок (локально мы её и тестировали).
- **windows-latest:** возможны проблемы с line endings (CRLF), длинными путями, fmt различиями. `.gitattributes` с `* text=auto eol=lf` решит CRLF; при необходимости добавить в Task 1 fixup commit и переpush.

Не идти дальше пока все 4 jobs не зелёные.

- [ ] **Step 10: Финальное обновление `plan/README.md` — отметить фазу как done**

Открыть `plan/README.md`, заменить:

```
- [~] Фаза 1 — Init (in progress)
```

на:

```
- [x] Фаза 1 — Init
```

- [ ] **Step 11: Commit финального статуса и push**

```bash
git add plan/README.md
git commit -m "chore(phase-1): mark Phase 1 complete

CI green on macos/ubuntu/windows-latest. Repo IGoRFonin/cchud (private)
contains skeleton + tests + CI + docs. Migration to public deferred to
Phase 9 (distribution).
"
git push
```

Verify: `gh run list --workflow=CI --limit 1 --json conclusion` → новый run для этого коммита тоже зелёный (можно дождаться через `gh run watch --exit-status` ещё раз).

- [ ] **Step 12: Final verification всей фазы**

```bash
# Локальное состояние
cargo build --release --locked && cargo test --locked && cargo clippy --locked -- -D warnings && cargo fmt --check

# Удалённое состояние
gh repo view IGoRFonin/cchud --json visibility,defaultBranchRef
gh run list --workflow=CI --limit 5 --json conclusion,headSha,event

# Git history
git log --oneline | head -10
```

Expected:
- 4 cargo команды зелёные.
- Repo private, default branch main (или master — не блокер).
- Минимум 2 успешных CI run'а (initial push + final commit).
- 6 коммитов: cargo bootstrap, skeleton, ci, docs, decisions, mark-complete.

## Verification (стандартный гейт + task-specific)

См. Step 12.

## Definition of Done

- [ ] Repo `IGoRFonin/cchud` существует, visibility = `PRIVATE`
- [ ] 6 коммитов Phase 1 запушены в `origin/main`
- [ ] `gh run list --workflow=CI` показывает успешные runs (`conclusion=success`) для последнего push'а на всех 3 OS
- [ ] `docs/DECISIONS.md` содержит запись о private-repo решении
- [ ] `plan/README.md` отмечает Фазу 1 как `[x]`
- [ ] Локально все 4 cargo-команды зелёные

## Files touched

- `docs/DECISIONS.md` (appended)
- `plan/README.md` (status update)
- (remote) `github.com/IGoRFonin/cchud` создан

## Risks & rollback

- **Имя занято:** см. Step 7 fallback (выбрать новое имя, обновить Cargo.toml/README/ATTRIBUTION). Маловероятно — Phase 0 проверила свободность 9 дней назад.
- **CI красный на windows-latest:** типично line endings или fmt-дифф. Добавить `.gitattributes`, переpush. Не считается окончанием Task 5 — продолжать пока не зелёный.
- **`gh auth` истёк:** `gh auth refresh -s repo,workflow`.
- **Юзер хочет public сразу:** `gh repo edit IGoRFonin/cchud --visibility public --accept-visibility-change-consequences` после Step 9. Обновить DECISIONS соответственно.
- **Что-то секретное случайно закоммичено:** до push (Step 7) — `git filter-branch` или `git filter-repo` чтобы переписать историю. После push — `gh repo delete IGoRFonin/cchud --yes` и начать заново.
- **Rollback (если репо создан, но решили откатить):** `gh repo delete IGoRFonin/cchud --yes`, `git remote remove origin`, `git reset --hard <commit-before-publish>`. Деструктивно — только с явным OK пользователя.
