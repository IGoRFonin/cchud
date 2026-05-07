# Task 6 — Worktree cluster (5 widgets)

**Files:**
- Create: `src/widgets/worktree.rs` (5 widgets: `Worktree`, `WorktreeMode`, `WorktreeName`, `WorktreeBranch`, `WorktreeOriginalBranch`)
- Modify: `src/widgets/mod.rs` (`pub mod worktree;`; 5 match-arms заменяют `Stub` на реальные impl)

## Goal

Кластер виджетов вокруг `payload.worktree: Option<Worktree>` (типизировано в T1).

| Widget | Source | Render |
|---|---|---|
| `Worktree` | `payload.worktree?.name` | `Some(name)` (контент-индикатор) |
| `WorktreeMode` | `payload.worktree.is_some()` | `Some("WT")` если активен; иначе None — флаг |
| `WorktreeName` | `payload.worktree?.name` | `Some(name.clone())` |
| `WorktreeBranch` | `payload.worktree?.branch` | `Some(branch.clone())` |
| `WorktreeOriginalBranch` | `payload.worktree?.original_branch` | `Some(b.clone())` |

`Worktree` (без суффикса) и `WorktreeName` рендерят одно и то же по сути, но это паритет с upstream API; не объединяем — пользователи могут конфигурировать любой.

## Inputs

- T1, T2, T3, T4, T5 закрыты.
- `payload.worktree` типизирован как `Option<Worktree>` со всеми полями `Option<String>`.
- `benches/samples/payload-synthetic-vim-worktree.json` создан в T1, содержит `worktree: { name, path, branch, original_cwd, original_branch }`.

---

- [ ] **Step 1: Написать failing-тесты**

Create `/Users/igor/mp/startup/cchud/src/widgets/worktree.rs`:

```rust
//! Worktree cluster — Phase 3 Task 6.
//!
//! Пять виджетов читают `payload.worktree: Option<Worktree>`. CC шлёт
//! поле только если активная сессия идёт в git-worktree (`git worktree add`).
//! Phase 0 семплы worktree не содержат — тесты используют synthetic-семпл
//! `payload-synthetic-vim-worktree.json` + struct-литералы.
//!
//! `Worktree` (без суффикса) и `WorktreeName` дают один и тот же контент;
//! upstream ccstatusline даёт оба, паритет требует обоих.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::widgets::{RenderContext, Widget};

pub struct Worktree;

impl Widget for Worktree {
    fn id(&self) -> &'static str {
        "Worktree"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let name = ctx.payload.worktree.as_ref()?.name.as_deref()?;
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }
}

pub struct WorktreeMode;

impl Widget for WorktreeMode {
    fn id(&self) -> &'static str {
        "WorktreeMode"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        if ctx.payload.worktree.is_some() {
            Some("WT".to_string())
        } else {
            None
        }
    }
}

pub struct WorktreeName;

impl Widget for WorktreeName {
    fn id(&self) -> &'static str {
        "WorktreeName"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let name = ctx.payload.worktree.as_ref()?.name.as_deref()?;
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }
}

pub struct WorktreeBranch;

impl Widget for WorktreeBranch {
    fn id(&self) -> &'static str {
        "WorktreeBranch"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let b = ctx.payload.worktree.as_ref()?.branch.as_deref()?;
        if b.is_empty() {
            None
        } else {
            Some(b.to_string())
        }
    }
}

pub struct WorktreeOriginalBranch;

impl Widget for WorktreeOriginalBranch {
    fn id(&self) -> &'static str {
        "WorktreeOriginalBranch"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let b = ctx
            .payload
            .worktree
            .as_ref()?
            .original_branch
            .as_deref()?;
        if b.is_empty() {
            None
        } else {
            Some(b.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::config::default_line;
    use crate::types::payload::{
        ModelInfo, StatusPayload, Worktree as WorktreePayload, Workspace,
    };

    const SYNTHETIC_SAMPLE: &str = include_str!(
        "../../benches/samples/payload-synthetic-vim-worktree.json"
    );

    fn payload_with_worktree(wt: Option<WorktreePayload>) -> StatusPayload {
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
            worktree: wt,
            vim: None,
            rate_limits: None,
            effort: None,
            thinking: None,
        }
    }

    fn full_worktree() -> WorktreePayload {
        WorktreePayload {
            name: Some("wt-feature".into()),
            path: Some("/tmp/wt-feature".into()),
            branch: Some("feature/x".into()),
            original_cwd: Some("/tmp/main".into()),
            original_branch: Some("main".into()),
        }
    }

    fn ctx_with<'a>(p: &'a StatusPayload, s: &'a crate::types::config::Settings) -> RenderContext<'a> {
        RenderContext::new(p, s)
    }

    // ─── Worktree (alias for name) ──────────────────────────────

    #[test]
    fn worktree_renders_name() {
        let p = payload_with_worktree(Some(full_worktree()));
        let s = default_line();
        assert_eq!(
            Worktree.render(&ctx_with(&p, &s)),
            Some("wt-feature".into())
        );
    }

    #[test]
    fn worktree_returns_none_without_field() {
        let p = payload_with_worktree(None);
        let s = default_line();
        assert_eq!(Worktree.render(&ctx_with(&p, &s)), None);
    }

    // ─── WorktreeMode ───────────────────────────────────────────

    #[test]
    fn worktree_mode_returns_wt_flag_when_active() {
        let p = payload_with_worktree(Some(full_worktree()));
        let s = default_line();
        assert_eq!(WorktreeMode.render(&ctx_with(&p, &s)), Some("WT".into()));
    }

    #[test]
    fn worktree_mode_returns_none_when_inactive() {
        let p = payload_with_worktree(None);
        let s = default_line();
        assert_eq!(WorktreeMode.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn worktree_mode_returns_wt_even_for_empty_worktree_object() {
        // worktree: {} — все поля None, но envelope-объект есть.
        // Mode реагирует на наличие объекта, а не на содержимое.
        let p = payload_with_worktree(Some(WorktreePayload {
            name: None,
            path: None,
            branch: None,
            original_cwd: None,
            original_branch: None,
        }));
        let s = default_line();
        assert_eq!(WorktreeMode.render(&ctx_with(&p, &s)), Some("WT".into()));
    }

    // ─── WorktreeName ───────────────────────────────────────────

    #[test]
    fn worktree_name_renders() {
        let p = payload_with_worktree(Some(full_worktree()));
        let s = default_line();
        assert_eq!(
            WorktreeName.render(&ctx_with(&p, &s)),
            Some("wt-feature".into())
        );
    }

    #[test]
    fn worktree_name_returns_none_without_name_field() {
        let mut wt = full_worktree();
        wt.name = None;
        let p = payload_with_worktree(Some(wt));
        let s = default_line();
        assert_eq!(WorktreeName.render(&ctx_with(&p, &s)), None);
    }

    // ─── WorktreeBranch ─────────────────────────────────────────

    #[test]
    fn worktree_branch_renders() {
        let p = payload_with_worktree(Some(full_worktree()));
        let s = default_line();
        assert_eq!(
            WorktreeBranch.render(&ctx_with(&p, &s)),
            Some("feature/x".into())
        );
    }

    #[test]
    fn worktree_branch_returns_none_without_branch() {
        let mut wt = full_worktree();
        wt.branch = None;
        let p = payload_with_worktree(Some(wt));
        let s = default_line();
        assert_eq!(WorktreeBranch.render(&ctx_with(&p, &s)), None);
    }

    // ─── WorktreeOriginalBranch ─────────────────────────────────

    #[test]
    fn worktree_original_branch_renders() {
        let p = payload_with_worktree(Some(full_worktree()));
        let s = default_line();
        assert_eq!(
            WorktreeOriginalBranch.render(&ctx_with(&p, &s)),
            Some("main".into())
        );
    }

    #[test]
    fn worktree_original_branch_returns_none_without_field() {
        let mut wt = full_worktree();
        wt.original_branch = None;
        let p = payload_with_worktree(Some(wt));
        let s = default_line();
        assert_eq!(WorktreeOriginalBranch.render(&ctx_with(&p, &s)), None);
    }

    // ─── Integration с synthetic-семплом ────────────────────────

    #[test]
    fn synthetic_sample_renders_full_cluster() {
        let p: StatusPayload =
            serde_json::from_str(SYNTHETIC_SAMPLE).expect("synthetic sample must parse");
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(Worktree.render(&ctx), Some("wt-feature".into()));
        assert_eq!(WorktreeMode.render(&ctx), Some("WT".into()));
        assert_eq!(WorktreeName.render(&ctx), Some("wt-feature".into()));
        assert_eq!(
            WorktreeBranch.render(&ctx),
            Some("feature/synthetic".into())
        );
        assert_eq!(WorktreeOriginalBranch.render(&ctx), Some("main".into()));
    }
}
```

- [ ] **Step 2: Запустить — должны упасть на компиляции**

```bash
cargo test --locked --lib widgets::worktree 2>&1 | head -10
```

Expected: `error[E0583]: file not found for module ...`.

- [ ] **Step 3: Подключить модуль + заменить 5 stub'ов в `build_one`**

Edit `src/widgets/mod.rs`:

Edit 1 (mod):
- `old_string`: `pub mod context;\npub mod model;\npub mod session;\npub mod static_text;\npub mod trivial;`
- `new_string`: `pub mod context;\npub mod model;\npub mod session;\npub mod static_text;\npub mod trivial;\npub mod worktree;`
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

Edit 2 (заменить 5 Stub-arm на реальные impl):
- `old_string`:
  ```rust
          WidgetConfig::Worktree => Box::new(Stub("Worktree")),
          WidgetConfig::WorktreeMode => Box::new(Stub("WorktreeMode")),
          WidgetConfig::WorktreeName => Box::new(Stub("WorktreeName")),
          WidgetConfig::WorktreeBranch => Box::new(Stub("WorktreeBranch")),
          WidgetConfig::WorktreeOriginalBranch => Box::new(Stub("WorktreeOriginalBranch")),
  ```
- `new_string`:
  ```rust
          // Phase 3 — Task 6 (worktree cluster):
          WidgetConfig::Worktree => Box::new(worktree::Worktree),
          WidgetConfig::WorktreeMode => Box::new(worktree::WorktreeMode),
          WidgetConfig::WorktreeName => Box::new(worktree::WorktreeName),
          WidgetConfig::WorktreeBranch => Box::new(worktree::WorktreeBranch),
          WidgetConfig::WorktreeOriginalBranch => Box::new(worktree::WorktreeOriginalBranch),
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

- [ ] **Step 4: Запустить тесты — должны быть зелёные**

```bash
cargo test --locked --lib widgets::worktree
```

Expected: 13 тестов passed (2 Worktree + 3 Mode + 2 Name + 2 Branch + 2 OriginalBranch + 1 Integration + 1 alias = sum 13).

Если `synthetic_sample_renders_full_cluster` падает — проверить, что `payload-synthetic-vim-worktree.json` валиден и парсится в `StatusPayload`. Проверить `cargo test --locked --lib types::payload::tests::parses_vim_and_worktree_envelope_fields`.

- [ ] **Step 5: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: всё зелёное. Тестов суммарно ≥102.

Возможные clippy:
- `pedantic::module_name_repetitions` на `worktree::Worktree`/`worktree::WorktreeMode` — это паритет с upstream API. Если жалуется — `#[allow(clippy::module_name_repetitions)]` на impl.
- `pedantic::no_effect_underscore_binding` — нет underscore.

- [ ] **Step 6: Verification — task-specific gate**

```bash
grep -c 'pub struct Worktree' src/widgets/worktree.rs
grep -c 'pub struct WorktreeMode' src/widgets/worktree.rs
grep -c 'pub struct WorktreeName' src/widgets/worktree.rs
grep -c 'pub struct WorktreeBranch' src/widgets/worktree.rs
grep -c 'pub struct WorktreeOriginalBranch' src/widgets/worktree.rs
grep -c 'worktree::Worktree' src/widgets/mod.rs
```

Expected:
```
1
1
1
1
1
≥1   (один или несколько worktree::Worktree* references)
```

- [ ] **Step 7: Commit**

```bash
git add src/widgets/worktree.rs src/widgets/mod.rs
git commit -m "feat(phase-3): T6 worktree cluster (5 widgets)

- Worktree: name (alias for WorktreeName, parity with upstream)
- WorktreeMode: 'WT' flag if any worktree object present
- WorktreeName: payload.worktree.name
- WorktreeBranch: payload.worktree.branch
- WorktreeOriginalBranch: payload.worktree.original_branch

All return None when payload.worktree is None or specific field missing.
WorktreeMode returns 'WT' even for empty {} object (presence flag).

Tests cover 12 unit + 1 synthetic-sample integration; payload-synthetic-
vim-worktree.json from T1 used as fixture.

Task 6/9 of Phase 3.
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

- [ ] `src/widgets/worktree.rs` создан, содержит 5 виджетов
- [ ] `Worktree` и `WorktreeName` рендерят `name` (паритет с upstream)
- [ ] `WorktreeMode` возвращает `"WT"` для любого присутствующего worktree-объекта (включая `{}`)
- [ ] Все виджеты graceful возвращают None для отсутствующих полей или `payload.worktree: None`
- [ ] `widgets::mod` подключает `pub mod worktree;` и инстанциирует 5 виджетов
- [ ] `cargo test --locked --lib widgets::worktree` — 13 тестов зелёные
- [ ] Integration-тест на synthetic-семпле валидирует все 5 виджетов в одном проходе
- [ ] Один commit `feat(phase-3): T6 worktree cluster ...`

## Files touched

- `src/widgets/worktree.rs` (created)
- `src/widgets/mod.rs` (modified)

## Risks & rollback

- **`WorktreeMode` ловит `worktree: {}` как активный**: документировано (presence flag). Если CC решит присылать пустой `worktree: {}` для не-worktree-сессий — поведение Mode = "WT" станет шумом. Сейчас (Phase 0 семплы) такого не наблюдается; mitigation — следить в Manual real-CC test (T9).
- **Имя `Worktree` widget vs тип `Worktree` (payload sub-type)**: в `tests` импорт `Worktree as WorktreePayload` решает конфликт; в `src/widgets/worktree.rs` ничего не сталкивается, потому что виджет в `widgets` модуле, payload-тип в `types::payload`.
- **`#[serde(rename_all = "kebab-case")]` rendering**: `WorktreeOriginalBranch` → `"worktree-original-branch"` в JSON. Проверим в T1 unit-тестах config (если не проверено там — добавить assertion в `parses_phase3_widget_kinds`).
- **Phase 5 (git widgets) нужен `Branch`**: упомянутый upstream-widget читает git, не payload.worktree.branch. Phase 5 добавит отдельный `Branch` от `gix`. Phase 3 виджет `WorktreeBranch` — read-only из payload, не конфликтует.
- **Rollback**: `git revert HEAD` — снимает кластер; Stub'ы восстанавливаются.
