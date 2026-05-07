# Task 7 — CustomCommand widget (subprocess + wait-timeout)

**Files:**
- Create: `src/widgets/custom_command.rs` (`CustomCommand` Widget impl + Unix-only tests)
- Modify: `src/widgets/mod.rs` (`pub mod custom_command;`; 1 match-arm заменяет `Stub` на реальный impl)

## Goal

Один виджет, но политически и инженерно нетривиальный — пользователь может задать произвольную команду в config, мы spawn'им subprocess с timeout'ом и берём stdout как сегмент statusline.

Контракт:
- argv-style spawn (`Command::new(&command).args(&args)`) — НИКАКОГО `sh -c`. Нет shell-injection через `text` / `args`.
- inherit env родителя (cchud) — конфиг пишет пользователь, контекст его. Документируется в README (T9).
- timeout default 200 ms (REQ-102), overridable через `params.timeout_ms`.
- На таймаут → kill child (через `wait_timeout::ChildExt::wait_timeout`), вернуть None.
- На non-zero exit → None молча. Phase 7 добавит `--verbose` диагностику; Phase 3 — silent.
- На spawn-fail (binary not found) → None.
- stdout → trim() → если пусто → None, иначе Some(string).
- stderr → discarded (`Stdio::null()` для child stderr).
- stdin → `Stdio::null()` (child не должен ждать input).

## Inputs

- T1, T2, T3, T4, T5, T6 закрыты.
- `wait-timeout = "0.2"` в `Cargo.toml` (T1).
- `WidgetConfig::CustomCommand { params: CustomCommandParams }` парсится; `timeout_ms` default 200.

---

- [ ] **Step 1: Написать failing-тесты**

Create `/Users/igor/mp/startup/cchud/src/widgets/custom_command.rs`:

```rust
//! CustomCommand widget — Phase 3 Task 7.
//!
//! Spawns a user-configured subprocess argv-style (без shell), читает
//! stdout с timeout'ом и возвращает trimmed-content. Любой негативный
//! сценарий → None молча (Phase 3 не диагностирует stderr).
//!
//! Security:
//! - Без `sh -c` → нет shell-injection.
//! - Inherit parent env → пользователь сам отвечает за безопасность
//!   команды; в README предупреждение про API-keys.
//! - Stdin = null → child не ждёт input.
//! - Stderr = null → не загрязняем stderr cchud.
//!
//! Платформа: Unix-only тесты под `#[cfg(unix)]`. Windows валиден на
//! компиляции, но behavioural-тесты (echo / sleep / false) отложены до
//! Phase 9 (distribution).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::Duration;

use wait_timeout::ChildExt;

use crate::types::config::CustomCommandParams;
use crate::widgets::{RenderContext, Widget};

pub struct CustomCommand {
    pub params: CustomCommandParams,
}

impl Widget for CustomCommand {
    fn id(&self) -> &'static str {
        "CustomCommand"
    }

    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        let mut child = Command::new(&self.params.command)
            .args(&self.params.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;

        let timeout = Duration::from_millis(self.params.timeout_ms);
        match child.wait_timeout(timeout).ok()? {
            None => {
                // Timeout — kill child, swallow result.
                let _ = child.kill();
                let _ = child.wait();
                None
            }
            Some(status) => {
                if !status.success() {
                    return None;
                }
                let mut out = String::new();
                child.stdout.as_mut()?.read_to_string(&mut out).ok()?;
                let trimmed = out.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            }
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::config::default_line;
    use crate::types::payload::{ModelInfo, StatusPayload, Workspace};

    fn empty_payload() -> StatusPayload {
        StatusPayload {
            session_id: "test".into(),
            model: ModelInfo {
                id: "claude-sonnet-4-6".into(),
                display_name: "Sonnet 4.6".into(),
            },
            workspace: Workspace {
                current_dir: "/tmp".into(),
                project_dir: None,
                added_dirs: None,
            },
            transcript_path: None,
            cwd: None,
            version: None,
            fast_mode: None,
            exceeds_200k_tokens: None,
            output_style: None,
            cost: None,
            context_window: None,
            worktree: None,
            vim: None,
            rate_limits: None,
            effort: None,
            thinking: None,
        }
    }

    fn ctx_with<'a>(p: &'a StatusPayload, s: &'a crate::types::config::Settings) -> RenderContext<'a> {
        RenderContext::new(p, s)
    }

    #[test]
    fn echo_returns_trimmed_stdout() {
        let p = empty_payload();
        let s = default_line();
        let w = CustomCommand {
            params: CustomCommandParams {
                command: "echo".into(),
                args: vec!["hello-cc".into()],
                timeout_ms: 1_000,
            },
        };
        assert_eq!(w.render(&ctx_with(&p, &s)), Some("hello-cc".into()));
    }

    #[test]
    fn echo_empty_returns_none() {
        let p = empty_payload();
        let s = default_line();
        // /bin/true exits 0 with empty stdout.
        let w = CustomCommand {
            params: CustomCommandParams {
                command: "true".into(),
                args: vec![],
                timeout_ms: 1_000,
            },
        };
        assert_eq!(w.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn nonzero_exit_returns_none() {
        let p = empty_payload();
        let s = default_line();
        // /usr/bin/false exits 1.
        let w = CustomCommand {
            params: CustomCommandParams {
                command: "false".into(),
                args: vec![],
                timeout_ms: 1_000,
            },
        };
        assert_eq!(w.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn missing_binary_returns_none() {
        let p = empty_payload();
        let s = default_line();
        let w = CustomCommand {
            params: CustomCommandParams {
                command: "/no/such/binary-xyz-12345".into(),
                args: vec![],
                timeout_ms: 1_000,
            },
        };
        assert_eq!(w.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn timeout_kills_child_and_returns_none() {
        let p = empty_payload();
        let s = default_line();
        // sleep 5 — но мы ждём только 50 ms.
        let w = CustomCommand {
            params: CustomCommandParams {
                command: "sleep".into(),
                args: vec!["5".into()],
                timeout_ms: 50,
            },
        };
        let start = std::time::Instant::now();
        let out = w.render(&ctx_with(&p, &s));
        let elapsed = start.elapsed();
        assert_eq!(out, None);
        // Должен выйти быстро (timeout 50 ms + kill); строго < 1 сек.
        assert!(
            elapsed < Duration::from_millis(1_000),
            "render did not respect timeout: elapsed = {elapsed:?}"
        );
    }

    #[test]
    fn trims_trailing_newline_and_whitespace() {
        let p = empty_payload();
        let s = default_line();
        // /bin/echo by default добавляет \n; trim удаляет его.
        // Также проверим pre/post whitespace через printf.
        let w = CustomCommand {
            params: CustomCommandParams {
                command: "printf".into(),
                args: vec!["  spaced  \n".into()],
                timeout_ms: 1_000,
            },
        };
        assert_eq!(w.render(&ctx_with(&p, &s)), Some("spaced".into()));
    }

    #[test]
    fn argv_avoids_shell_interpretation() {
        let p = empty_payload();
        let s = default_line();
        // Если бы был sh -c, ; разделил бы команды и `whoami` бы выполнилось.
        // С argv-style "echo" получает буквальный "$(whoami); echo INJECTED"
        // как один аргумент.
        let w = CustomCommand {
            params: CustomCommandParams {
                command: "echo".into(),
                args: vec!["$(whoami); echo INJECTED".into()],
                timeout_ms: 1_000,
            },
        };
        let out = w.render(&ctx_with(&p, &s)).unwrap();
        assert!(out.contains("$(whoami)"), "expected literal $(whoami), got {out:?}");
        assert!(!out.contains("INJECTED") || out.contains("INJECTED") && out.starts_with("$(whoami); echo INJECTED"),
            "expected no shell interpretation");
    }
}
```

**Note про `argv_avoids_shell_interpretation`**: проверка проще, если переписать как `assert_eq!(out, "$(whoami); echo INJECTED")` — тогда понятно, что shell не запускался. Замени `assert!`-цепочку на:

```rust
        let out = w.render(&ctx_with(&p, &s)).unwrap();
        assert_eq!(out, "$(whoami); echo INJECTED");
```

Это чище.

- [ ] **Step 2: Запустить — должны упасть на компиляции**

```bash
cargo test --locked --lib widgets::custom_command 2>&1 | head -10
```

Expected: `error[E0583]: file not found for module ...` — модуль не подключён.

- [ ] **Step 3: Подключить модуль + заменить 1 stub в `build_one`**

Edit `src/widgets/mod.rs`:

Edit 1 (mod):
- `old_string`: `pub mod context;\npub mod model;\npub mod session;\npub mod static_text;\npub mod trivial;\npub mod worktree;`
- `new_string`: `pub mod context;\npub mod custom_command;\npub mod model;\npub mod session;\npub mod static_text;\npub mod trivial;\npub mod worktree;`
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

Edit 2:
- `old_string`: `WidgetConfig::CustomCommand { .. } => Box::new(Stub("CustomCommand")),`
- `new_string`:
  ```rust
          // Phase 3 — Task 7 (custom-command):
          WidgetConfig::CustomCommand { params } => Box::new(custom_command::CustomCommand {
              params: params.clone(),
          }),
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

Edit 3 (удалить ставший ненужным `Stub` struct и его использование, ЕСЛИ все 22 stub-arm заменены к этому моменту):

Сначала проверить, что все stub-arms заменены:
```bash
grep -c 'Stub(' src/widgets/mod.rs
```

Expected: `1` (только в `Stub` struct definition + 0 references). Если grep даёт `2+` — некий stub-arm ещё остался; проверь и закрой соответствующей задачей.

Если все 22 заменены — удалить `Stub` struct:
- `old_string`:
  ```rust
  /// Stub Widget — placeholder для variants, чьи impl ещё не написаны
  /// (Phase 3: вытесняется match-arms по мере роста кластеров T2–T7).
  struct Stub(&'static str);
  impl Widget for Stub {
      fn id(&self) -> &'static str {
          self.0
      }
      fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
          None
      }
  }

  ```
- `new_string`: `` (пустая строка)
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

И комментарии-строки с упоминанием stub'ов чистим:
- `old_string`: `// Phase 3 stubs — заменяются на реальные impl в T3–T7:` → удалить эту строку (или заменить на `// Phase 3 — все 23 виджета подключены:`).

- [ ] **Step 4: Запустить тесты — должны быть зелёные на Unix**

```bash
cargo test --locked --lib widgets::custom_command
```

Expected (Linux/macOS): 7 тестов passed.

```
test widgets::custom_command::tests::echo_returns_trimmed_stdout ... ok
test widgets::custom_command::tests::echo_empty_returns_none ... ok
test widgets::custom_command::tests::nonzero_exit_returns_none ... ok
test widgets::custom_command::tests::missing_binary_returns_none ... ok
test widgets::custom_command::tests::timeout_kills_child_and_returns_none ... ok
test widgets::custom_command::tests::trims_trailing_newline_and_whitespace ... ok
test widgets::custom_command::tests::argv_avoids_shell_interpretation ... ok
```

На Windows тесты пропускаются через `#[cfg(all(test, unix))]`. Компиляция самого виджета должна пройти.

- [ ] **Step 5: Проверить, что Windows-компиляция не сломана**

Если есть локально WSL/Windows-runner — `cargo build --release --locked --target x86_64-pc-windows-gnu`. Если нет — положиться на CI matrix.

Compiletime check: `cargo check --locked` — exit 0.

- [ ] **Step 6: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: всё зелёное. Тестов суммарно ≥109 (102 после T6 + 7 widgets::custom_command).

Возможные clippy-замечания:
- `nursery::redundant_pub_crate` — если `pub use` лишний. Игнорируем.
- `pedantic::struct_field_names` на `CustomCommand { params: CustomCommandParams }` — внутри struct поле названо `params`, лишний namespace в имени поля не дублируется (struct имеет только одно поле). ОК.
- `unwrap_used = "deny"` и `expect_used = "deny"` стоят `#![deny(...)]` первой строкой файла. **Перепроверь:** в impl-коде НЕ должно быть `unwrap()`/`expect()` — только `?` chains:

```bash
grep -nE 'unwrap\(\)|\.expect\(' src/widgets/custom_command.rs
```

Expected: матчи только в `mod tests` (там `#![allow(clippy::unwrap_used, clippy::expect_used)]`). Если в impl-коде `expect/unwrap` есть — переписать на `?`.

```bash
cargo clippy --locked -- -D warnings
```
Expected: exit 0.

- [ ] **Step 7: Verification — task-specific gate**

```bash
grep -c 'pub struct CustomCommand' src/widgets/custom_command.rs
grep -c 'wait_timeout::ChildExt' src/widgets/custom_command.rs
grep -c 'Stdio::null' src/widgets/custom_command.rs
grep -c 'Stdio::piped' src/widgets/custom_command.rs
grep -c 'custom_command::CustomCommand' src/widgets/mod.rs
grep -c '^struct Stub' src/widgets/mod.rs
```

Expected:
```
1
1
2    (stdin null + stderr null)
1    (stdout piped)
1
0    (Stub удалён)
```

- [ ] **Step 8: Commit**

```bash
git add src/widgets/custom_command.rs src/widgets/mod.rs
git commit -m "feat(phase-3): T7 CustomCommand widget — subprocess + wait-timeout

Spawns argv-style subprocess (no sh -c → no shell-injection), pipes
stdout, kills on timeout (default 200ms, configurable), discards
stderr. Returns trimmed stdout; None for: spawn-fail, non-zero exit,
timeout, empty trimmed stdout.

Inherits parent env (documented in README): user owns command safety;
opt-in env-allowlist + sandboxing deferred to Phase 7.

Tests Unix-only via #[cfg(all(test, unix))]: echo/true/false/sleep/
printf + missing binary + argv-not-shell. Windows compile-checked,
behavioural tests deferred to Phase 9.

All 23 Phase 3 widget variants now have real impls; Stub helper
removed from widgets/mod.rs.

Task 7/9 of Phase 3.
"
```

## Verification (стандартный гейт)

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

## Definition of Done

- [ ] `src/widgets/custom_command.rs` создан с `CustomCommand` Widget impl
- [ ] Argv-style spawn (`Command::new + .args()`) — НЕ `sh -c`
- [ ] `Stdio::null()` для stdin и stderr child'а; `Stdio::piped()` для stdout
- [ ] `wait_timeout::ChildExt::wait_timeout` использован для timeout-driven kill
- [ ] Default timeout = 200 ms (через `CustomCommandParams::timeout_ms`)
- [ ] Все негативные пути (timeout/non-zero/spawn-fail/empty stdout) → None silent
- [ ] 7 unit-тестов под `#[cfg(all(test, unix))]`: echo / empty-stdout / non-zero / missing-binary / timeout / trim / argv-not-shell
- [ ] `widgets::mod` подключает `pub mod custom_command;` и инстанциирует виджет
- [ ] Все 23 Stub-arm заменены реальными impls; `Stub` struct удалён
- [ ] CI matrix зелёный (Linux/macOS — тесты, Windows — компиляция)
- [ ] Один commit `feat(phase-3): T7 CustomCommand widget ...`

## Files touched

- `src/widgets/custom_command.rs` (created)
- `src/widgets/mod.rs` (modified, +1 mod, +1 match-arm, −Stub)

## Risks & rollback

- **`wait_timeout` API изменился между 0.2.x релизами**: lock файл фиксирует версию. Если minor bump пришёл — переcмотреть. На 2026-04 0.2.0 — последний релиз, API стабилен.
- **`sleep 5` тест долго runner-killed на medium-load CI**: timeout 50 ms должен убивать за ≪ 1 сек. Assert `elapsed < 1s` ловит.
- **Windows `wait_timeout` использует `WaitForSingleObject`**: компиляция OK; behavioural тесты под `#[cfg(unix)]`, Windows прошлая часть CI matrix только проверяет compile.
- **`echo` встроенная команда vs. `/bin/echo`**: `Command::new("echo")` ищет в PATH; первый бинарь — `/bin/echo` (не shell builtin), trailing newline присутствует, `trim()` справляется.
- **`printf` отсутствует в minimal `bin`-окружении**: если CI runner не имеет `/usr/bin/printf` — заменить тест `trims_trailing_newline_and_whitespace` на `bash -c "echo -n trimmed"` (но тогда мы используем shell — против политики). Альтернатива: использовать `echo "spaced"` (echo всегда дописывает \n, trim уберёт). Если `printf` падает — переписать тест на `echo`.
- **CustomCommand subprocess в hyperfine bench (T8) превышает 5 ms budget**: spec говорит "warning >5 ms with CustomCommand"; T8 решает это документацией.
- **Process inherits sensitive env (API keys)**: README предупреждение в T9; opt-in env-allowlist — Phase 7. Не блокер для Phase 3.
- **`expect_used = "deny"` срабатывает на `child.stdout.as_mut()?`**: `as_mut()` возвращает `Option<&mut ...>`, мы используем `?` — не `expect()`. ОК.
- **Rollback**: `git revert HEAD` — снимает виджет, возвращает Stub-arm + Stub struct.
