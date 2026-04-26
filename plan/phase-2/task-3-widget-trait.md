# Task 3 — Widget trait + RenderContext

**Files:**
- Create: `src/widgets/mod.rs`
- Modify: `src/main.rs` (добавить `mod widgets;`)

## Goal

Определить ядро виджет-системы: `Widget` trait, `RenderContext`, `build_widgets` функцию-фабрику. Никаких implementations пока — только API. Тестирование trait'а — через статический контракт (компилируется = ОК); реальный виджет (Model) появляется в Task 4.

## Inputs

- Task 2 закрыта (`StatusPayload` импортируется как `crate::types::payload::StatusPayload`).
- `src/main.rs` содержит `mod types;`.

---

- [ ] **Step 1: Создать `src/widgets/mod.rs` с trait и контекстом**

```rust
//! Widget trait, render context, and registry/factory for widgets.
//!
//! The trait is small on purpose: each widget gets `RenderContext`
//! (immutable view of payload + settings) and returns `Option<String>`
//! (None = "nothing to show", filtered out by the renderer).
//!
//! Phase 5 will add `git: OnceCell<Option<GitInfo>>` to `RenderContext`.
//! Phase 6 will add `transcript: OnceCell<Option<TranscriptCache>>`.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::types::payload::StatusPayload;

pub trait Widget: Send + Sync {
    fn id(&self) -> &'static str;
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String>;
}

pub struct RenderContext<'a> {
    pub payload: &'a StatusPayload,
    // settings: &'a Settings — добавится в Task 6 (после появления Settings типа в Task 5).
}

impl<'a> RenderContext<'a> {
    #[must_use]
    pub fn new(payload: &'a StatusPayload) -> Self {
        Self { payload }
    }
}

/// Build the list of widgets. In Task 4 returns a hardcoded `[Model]`.
/// In Task 6 (after `Settings` exists) it will iterate `settings.lines[0].widgets`.
#[must_use]
pub fn build_widgets() -> Vec<Box<dyn Widget>> {
    Vec::new()
}
```

**Note про дизайн-эволюцию:** `RenderContext::new` в Task 6 расширится до `new(payload, settings)` — это **breaking change** в API, но зона использования — наш собственный crate, рефактор тривиален. Альтернатива — сразу принимать `Option<&Settings>` — добавляет мусор в API на ровном месте. Принимаем текущий минимум.

- [ ] **Step 2: Подключить `mod widgets;` в `src/main.rs`**

Найти в `src/main.rs` строку `mod types;` и добавить ниже:

```rust
mod types;
mod widgets;
```

Edit tool:
- `old_string`: `mod types;\n`
- `new_string`: `mod types;\nmod widgets;\n`
- `file_path`: `/Users/igor/mp/startup/cchud/src/main.rs`

- [ ] **Step 3: Compile check**

```bash
cargo build --release --locked
```

Expected: zero errors. Возможны warnings от clippy nursery про `must_use_candidate` или похожее на `RenderContext::new` (мы уже поставили `#[must_use]`, но nursery может ругаться на сам метод). Если warning → ОК на этом этапе. Если **error** на `unused_imports` для `StatusPayload` — это ожидаемо если nightly-feature `unused_imports = "deny"` включён где-то; Phase 1 не включал. Должно компилиться чисто.

- [ ] **Step 4: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: тесты без изменений зелёные (никаких новых тестов в этой задаче — `Widget` trait тестируется через интеграционный smoke в Task 4). Clippy чист.

Если clippy ругается на `clippy::dead_code` для `build_widgets` (не вызывается из main) — это пройдёт после Task 4. На текущем этапе нужен `#[allow(dead_code)]` на `build_widgets`:

```rust
#[allow(dead_code)]
#[must_use]
pub fn build_widgets() -> Vec<Box<dyn Widget>> {
    Vec::new()
}
```

То же касается `RenderContext::new` — добавить `#[allow(dead_code)]` если clippy ругается. Удалим эти аннотации в Task 4 когда они начнут использоваться.

Если clippy nursery ругается на `clippy::missing_const_for_fn` для `RenderContext::new` — добавить `const`:
```rust
pub const fn new(payload: &'a StatusPayload) -> Self {
    Self { payload }
}
```

- [ ] **Step 5: Verification — task-specific gate**

```bash
test -f src/widgets/mod.rs && echo "widgets/mod.rs ok"
grep -q 'pub trait Widget' src/widgets/mod.rs && echo "Widget trait ok"
grep -q 'pub struct RenderContext' src/widgets/mod.rs && echo "RenderContext ok"
grep -q 'pub fn build_widgets' src/widgets/mod.rs && echo "build_widgets ok"
grep -q '^mod widgets;' src/main.rs && echo "main wired ok"
```

Expected output:
```
widgets/mod.rs ok
Widget trait ok
RenderContext ok
build_widgets ok
main wired ok
```

- [ ] **Step 6: Commit**

```bash
git add src/widgets/mod.rs src/main.rs
git commit -m "feat(phase-2): widget trait + RenderContext skeleton

Define core widget API:
- trait Widget { id(), render(&RenderContext) -> Option<String> }
- RenderContext { payload } — settings/git/transcript added in
  later tasks/phases.
- build_widgets() — empty Vec for now, hardcoded list in Task 4,
  config-driven in Task 6.

Module-level deny(unwrap_used, expect_used) enabled — hot-path
discipline before any real code lands.

Task 3/10 of Phase 2.
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

- [ ] `src/widgets/mod.rs` создан, содержит `pub trait Widget`, `pub struct RenderContext`, `pub fn build_widgets`
- [ ] `RenderContext` принимает `&'a StatusPayload` (settings добавится в Task 6)
- [ ] Файл начинается с `#![deny(clippy::unwrap_used, clippy::expect_used)]`
- [ ] `src/main.rs` содержит `mod widgets;`
- [ ] `cargo build --release --locked` зелёный
- [ ] `cargo clippy --locked -- -D warnings` зелёный
- [ ] `cargo test --locked` зелёный (количество тестов без изменений с Task 2)
- [ ] Один коммит `feat(phase-2): widget trait + RenderContext skeleton`

## Files touched

- `src/widgets/mod.rs` (created)
- `src/main.rs` (modified — `mod widgets;`)

## Risks & rollback

- **Clippy nursery вытаскивает `must_use_candidate` / `missing_const_for_fn`**: добавить соответствующие аннотации (см. Step 4 fallback). Не отключать nursery глобально.
- **`dead_code` warning** на `build_widgets`/`RenderContext::new`: временный `#[allow(dead_code)]`, снимется в Task 4.
- **`Send + Sync` bound на `Widget` ломает что-то будущее**: Phase 7+ если виджет захочет хранить `Rc<...>` — придётся менять trait. Пока единственный widget (`Model`) в Task 4 будет stateless → `Send + Sync` автоматически.
- **Rollback**: `git revert HEAD` — изолировано в одном коммите.
