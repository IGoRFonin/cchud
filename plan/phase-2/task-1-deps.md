# Task 1 — Dependency adjustments

**Files:**
- Modify: `Cargo.toml` (remove `lexopt`, add `serial_test` + `tempfile` to dev-dependencies)
- Modify: `Cargo.lock` (auto-regenerated)

## Goal

Подготовить `Cargo.toml` к Phase 2: убрать неиспользуемый `lexopt` (Phase 2 использует ручной `match args.first()` в `main.rs`), добавить dev-deps `serial_test` (для последовательных install-тестов) и `tempfile` (для изолированного HOME в install-тестах). Per-module lint tightening (`unwrap_used = "deny"`) делается в задачах создания модулей (T3/T4/T6), не здесь.

## Inputs

- Phase 1 закрыта; ветка `master`, рабочее дерево чистое (`git status` пусто).
- Текущий `Cargo.toml` содержит `lexopt = "0.3"` в `[dependencies]` (Phase 1 task-1 артефакт).
- `cargo build --release --locked` локально зелёный.

---

- [ ] **Step 1: Удалить `lexopt` из `[dependencies]`**

В `Cargo.toml` найти и удалить строку:
```toml
lexopt = "0.3"
```

Edit tool:
- `old_string`: `lexopt = "0.3"\n`
- `new_string`: ``
- `file_path`: `/Users/igor/mp/startup/cchud/Cargo.toml`

Verify: `grep -c lexopt Cargo.toml` → 0.

- [ ] **Step 2: Добавить `serial_test` и `tempfile` в `[dev-dependencies]`**

В `Cargo.toml` секцию `[dev-dependencies]` дополнить:

```toml
[dev-dependencies]
insta = { version = "1", features = ["json"] }
assert_cmd = "2"
predicates = "3"
serial_test = "3"
tempfile = "3"
```

Edit tool: заменить блок `[dev-dependencies] ... predicates = "3"` целиком. Конкретные строки можно увидеть через Read `Cargo.toml`.

Verify: `grep -E '^(serial_test|tempfile) ' Cargo.toml` → 2 строки.

- [ ] **Step 3: Регенерировать `Cargo.lock`**

```bash
cargo build --release --locked
```

Expected: `--locked` упадёт с ошибкой "Cargo.lock needs to be updated" — это ожидаемо после изменения deps.

Если упало — снять `--locked`:
```bash
cargo build --release
```

Expected: build тянет `serial_test 3.x` и `tempfile 3.x` + transitive (lazy_static, fastrand, rustix). Build time +5–15 сек. `Cargo.lock` обновлён.

- [ ] **Step 4: Повторный build с `--locked` для подтверждения чистоты**

```bash
cargo build --release --locked
```

Expected: no-op, exit 0, никакой работы (lock актуален). Если падает — `Cargo.lock` не сохранился; повторить Step 3 без `--locked` и проверить `git status` что `Cargo.lock` модифицирован.

- [ ] **Step 5: Standard gate (build + test + clippy + fmt)**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все 4 команды exit 0. `cargo test` запустит существующие Phase 1 snapshot-тесты (12 шт.) — зелёные.

Если clippy ругается — Phase 1 работал, мы только удалили deps; маловероятно. Если падает на `unused_imports` где-то — найти место и удалить лишний `use` (Phase 1 не использовал lexopt напрямую, но проверь `src/main.rs` на всякий случай).

- [ ] **Step 6: Verification — task-specific gate**

```bash
grep -c '^lexopt' Cargo.toml
grep -c '^serial_test' Cargo.toml
grep -c '^tempfile' Cargo.toml
cargo metadata --format-version 1 --locked | grep -c '"name":"lexopt"'
cargo metadata --format-version 1 --locked | grep -c '"name":"serial_test"'
```

Expected output:
```
0
1
1
0
1
```

(`lexopt` ушёл из manifest и из dependency tree; `serial_test` появился в обоих.)

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore(phase-2): swap lexopt → serial_test + tempfile (dev)

Phase 2 uses manual arg dispatch (match args.first()) in main.rs,
lexopt unused. Add serial_test + tempfile for upcoming install
tests with HOME override.

Task 1/10 of Phase 2.
"
```

Verify: `git log --oneline | head -1` показывает новый commit с префиксом `chore(phase-2):`.

## Verification (стандартный гейт)

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

## Definition of Done

- [ ] `Cargo.toml`: нет `lexopt`, есть `serial_test = "3"` и `tempfile = "3"` в `[dev-dependencies]`
- [ ] `Cargo.lock` обновлён (содержит `serial_test`, не содержит `lexopt`)
- [ ] `cargo build --release --locked` зелёный
- [ ] `cargo test --locked` зелёный (12 Phase 1 snapshot-тестов)
- [ ] `cargo clippy --locked -- -D warnings` зелёный
- [ ] `cargo fmt --check` зелёный
- [ ] Один коммит `chore(phase-2): swap lexopt → serial_test + tempfile (dev)`

## Files touched

- `Cargo.toml` (modified)
- `Cargo.lock` (auto-regenerated)

## Risks & rollback

- **`serial_test` или `tempfile` не резолвятся на MSRV 1.85**: маловероятно (обе зрелые crate). Если падает — поднять MSRV до 1.87 (есть локально через brew) и обновить `rust-version` в `Cargo.toml` + `rust-toolchain.toml`. Записать в `docs/DECISIONS.md`.
- **Cargo.lock конфликт** (если кто-то пушил параллельно): не релевантно (single-developer проект).
- **Rollback**: `git revert HEAD` или `git reset --hard HEAD~1` — изменения только в Cargo.toml/Cargo.lock, безопасно.
