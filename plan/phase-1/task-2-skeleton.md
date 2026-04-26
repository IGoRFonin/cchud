# Task 2 — Skeleton main.rs + Snapshot tests

**Files:**
- Modify: `src/main.rs` (replace hello-world with skeleton)
- Create: `tests/snapshots.rs`

## Goal

Заменить hello-world на skeleton, который читает payload Claude Code из stdin и печатает byte count. Покрыть всеми 12 payload-семплами из Фазы 0 через интеграционный тест на `assert_cmd`. В Фазе 2 этот тест эволюционирует в `insta::assert_snapshot!` с реальным rendering'ом.

## Inputs

- Task 1 завершён: `Cargo.toml` с deps, `cargo build` работает.
- `benches/samples/*.json` — 12 payload-файлов из Phase 0.
- `assert_cmd = "2"` уже в `[dev-dependencies]` (из Task 1 Cargo.toml).

---

- [ ] **Step 1: Написать failing test**

Создать `tests/snapshots.rs`:

```rust
//! Integration tests: skeleton reader.
//!
//! Phase 1 — verifies binary accepts stdin payloads from Phase 0 fixtures
//! and produces non-empty stdout with zero exit code. In Phase 2 this is
//! replaced by `insta::assert_snapshot!` with real rendered output.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use assert_cmd::Command;
use std::{error::Error, fs};

#[test]
fn renders_skeleton_for_each_phase0_sample() -> Result<(), Box<dyn Error>> {
    let mut count = 0;
    for entry in fs::read_dir("benches/samples")? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "json") {
            let payload = fs::read_to_string(&path)?;
            let output = Command::cargo_bin("cchud")?
                .write_stdin(payload.clone())
                .output()?;

            assert!(
                output.status.success(),
                "non-zero exit on {path:?}: stderr={}",
                String::from_utf8_lossy(&output.stderr),
            );
            assert!(
                !output.stdout.is_empty(),
                "empty stdout on {path:?}",
            );

            let stdout = String::from_utf8(output.stdout)?;
            assert!(
                stdout.contains("cchud (skeleton)"),
                "stdout missing skeleton marker for {path:?}: {stdout}",
            );
            assert!(
                stdout.contains(&payload.len().to_string()),
                "stdout missing byte count for {path:?}: {stdout}",
            );

            count += 1;
        }
    }
    assert!(count >= 12, "expected ≥12 payload samples, found {count}");
    Ok(())
}
```

- [ ] **Step 2: Запустить тест — ожидать FAIL**

```bash
cargo test --locked --test snapshots
```

Expected: FAIL. `src/main.rs` всё ещё hello-world, выводит `Hello, world!`. Тест падает на `assert!(stdout.contains("cchud (skeleton)"))`. Output:
```
thread 'renders_skeleton_for_each_phase0_sample' panicked at '...stdout missing skeleton marker for "benches/samples/payload-...": Hello, world!\n', tests/snapshots.rs:NN
```

- [ ] **Step 3: Реализовать skeleton main.rs**

Перезаписать `src/main.rs`:

```rust
//! cchud — Fast Rust statusline for Claude Code CLI.
//!
//! Phase 1 skeleton: read JSON payload from stdin, print byte count.
//! Real widget pipeline lands in Phase 2.

use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let len = input.len();
    println!("cchud (skeleton) | input bytes: {len}");
    Ok(())
}
```

- [ ] **Step 4: Запустить тест — ожидать PASS**

```bash
cargo test --locked --test snapshots
```

Expected: PASS. Output:
```
running 1 test
test renders_skeleton_for_each_phase0_sample ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; ...
```

Note: один integration test, итерирующий все 12 семплов внутри. Падение на любом семпле = падение всего теста с указанием конкретного path. Если хочешь увидеть отдельные test cases в выводе — это перейдёт на `insta` в Phase 2 (там per-snapshot файлы).

- [ ] **Step 5: Проверить вручную на одном из xlarge семплов**

```bash
cargo build --release --locked
cat benches/samples/payload-cchud-opus-xlarge.json | ./target/release/cchud
```

Expected output:
```
cchud (skeleton) | input bytes: 1124
```

(Точное число — длина файла; должно совпасть с `wc -c < benches/samples/payload-cchud-opus-xlarge.json`.)

- [ ] **Step 6: Проверить standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все четыре зелёные. clippy на skeleton main.rs не должен ругаться: модульный docstring есть, нет `unwrap`/`expect`, format string использует inline `{len}`.

Если clippy ругается на `must_use_candidate` или подобное — добавить `#[allow(...)]` инлайн с комментарием почему.

- [ ] **Step 7: Commit**

```bash
git add src/main.rs tests/snapshots.rs
git commit -m "feat(phase-1): skeleton reader + snapshot tests on phase-0 payloads

src/main.rs:
- read stdin to String
- print 'cchud (skeleton) | input bytes: N'
- exit 0 on success

tests/snapshots.rs:
- iterate benches/samples/*.json (12 payloads from Phase 0)
- assert: exit 0, stdout non-empty, contains 'cchud (skeleton)' and byte count

Phase 2 will replace skeleton with real widget pipeline and migrate to
insta::assert_snapshot! per-payload.

Task 2/5 of Phase 1.
"
```

## Verification (стандартный гейт + task-specific)

```bash
# Standard gate
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check

# Task-specific
for f in benches/samples/*.json; do
  out=$(cat "$f" | ./target/release/cchud)
  bytes=$(wc -c < "$f" | tr -d ' ')
  expected="cchud (skeleton) | input bytes: $bytes"
  if [ "$out" = "$expected" ]; then
    echo "OK: $(basename "$f")"
  else
    echo "FAIL: $(basename "$f"): expected '$expected', got '$out'"
    exit 1
  fi
done
```

Expected: 12 строк `OK: payload-...json`.

## Definition of Done

- [ ] `cargo test --locked` зелёный (1 интеграционный тест, итерирует ≥12 семплов)
- [ ] `cargo clippy --locked -- -D warnings` чисто
- [ ] Ручной прогон на любом семпле выводит `cchud (skeleton) | input bytes: N` где N = размер файла
- [ ] Один коммит с префиксом `feat(phase-1): skeleton reader`

## Files touched

- `src/main.rs` (modified — replaces hello-world)
- `tests/snapshots.rs` (created)

## Risks & rollback

- **`benches/samples/` пустой или содержит < 12 файлов:** ассерт `count >= 12` упадёт. Phase 0 артефакт — должен быть. Если нет — вернуться к Phase 0 Task 4 и восстановить семплы.
- **Pedantic ругается на тестовый код:** в файле уже `#![allow(clippy::unwrap_used, clippy::expect_used)]`. Если ругается на что-то другое (`uninlined_format_args` etc.) — поправить инлайн.
- **`assert_cmd` не находит binary:** означает что `cargo build` не отработал перед тестом (cargo обычно сам билдит). Workaround: `cargo build --release` явно перед `cargo test`.
- **Rollback:** `git checkout HEAD~1 -- src/main.rs` (вернуть hello-world); `rm tests/snapshots.rs`.
