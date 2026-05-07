# Task 2 — Static cluster (CustomText, CustomSymbol, Link)

**Files:**
- Create: `src/widgets/static_text.rs` (3 widget impl: `CustomText`, `CustomSymbol`, `Link`)
- Modify: `src/widgets/mod.rs` (`pub mod static_text;`; 3 match-arms заменяют `Stub` на реальные impl)

## Goal

Первый кластер виджетов — самый простой: только статический текст из конфига, без чтения payload.

| Widget | Render | Edge |
|---|---|---|
| `CustomText` | `Some(params.text.clone())` | Пустая строка → None (Plain renderer фильтрует пустые сегменты) |
| `CustomSymbol` | `Some(params.symbol.clone())` | Пустая строка → None |
| `Link` | OSC 8: `\x1b]8;;{url}\x1b\\{label_or_url}\x1b]8;;\x1b\\` | Phase 3 emit без detect terminal capability; пустой `url` → None. `label` отсутствует → используем `url` как label. |

Сознательно НЕ включаем (это Phase 4 / Phase 7):
- color/bold/background стилизацию
- detect-логику OSC 8 (рендерим всегда; при не-поддерживающем терминале пользователь увидит "сырые" escape-байты — это документируется в README в T9).

## Inputs

- T1 закрыт: `WidgetConfig::CustomText { params: CustomTextParams }`, `CustomSymbol`, `Link` объявлены и парсятся; `Stub` подключён в `build_one`.
- `cargo test --locked --lib types::config::tests::parses_phase3_widget_kinds` зелёный.

---

- [ ] **Step 1: Написать failing-тесты для всего кластера**

Create `/Users/igor/mp/startup/cchud/src/widgets/static_text.rs`:

```rust
//! Static-text cluster — Phase 3 Task 2.
//!
//! Виджеты этого модуля не читают payload; они рендерят буквальный текст
//! из конфига. `Link` использует OSC 8 hyperlink escape-последовательность
//! (без detect terminal capability — Phase 7 добавит graceful fallback).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::types::config::{CustomSymbolParams, CustomTextParams, LinkParams};
use crate::widgets::{RenderContext, Widget};

pub struct CustomText {
    pub params: CustomTextParams,
}

impl Widget for CustomText {
    fn id(&self) -> &'static str {
        "CustomText"
    }
    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        if self.params.text.is_empty() {
            None
        } else {
            Some(self.params.text.clone())
        }
    }
}

pub struct CustomSymbol {
    pub params: CustomSymbolParams,
}

impl Widget for CustomSymbol {
    fn id(&self) -> &'static str {
        "CustomSymbol"
    }
    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        if self.params.symbol.is_empty() {
            None
        } else {
            Some(self.params.symbol.clone())
        }
    }
}

pub struct Link {
    pub params: LinkParams,
}

impl Widget for Link {
    fn id(&self) -> &'static str {
        "Link"
    }
    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        if self.params.url.is_empty() {
            return None;
        }
        let label = self.params.label.as_deref().unwrap_or(&self.params.url);
        // OSC 8 hyperlink: ESC ] 8 ; ; URL ST  TEXT  ESC ] 8 ; ; ST
        // ST (string terminator) = ESC \ (0x1b 0x5c).
        Some(format!(
            "\x1b]8;;{url}\x1b\\{label}\x1b]8;;\x1b\\",
            url = self.params.url,
        ))
    }
}

#[cfg(test)]
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

    #[test]
    fn custom_text_renders_literal() {
        let p = empty_payload();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let w = CustomText {
            params: CustomTextParams { text: "hello".into() },
        };
        assert_eq!(w.render(&ctx), Some("hello".into()));
    }

    #[test]
    fn custom_text_returns_none_for_empty() {
        let p = empty_payload();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let w = CustomText {
            params: CustomTextParams { text: String::new() },
        };
        assert_eq!(w.render(&ctx), None);
    }

    #[test]
    fn custom_symbol_renders_literal() {
        let p = empty_payload();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let w = CustomSymbol {
            params: CustomSymbolParams { symbol: "★".into() },
        };
        assert_eq!(w.render(&ctx), Some("★".into()));
    }

    #[test]
    fn custom_symbol_returns_none_for_empty() {
        let p = empty_payload();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let w = CustomSymbol {
            params: CustomSymbolParams { symbol: String::new() },
        };
        assert_eq!(w.render(&ctx), None);
    }

    #[test]
    fn link_emits_osc8_with_label() {
        let p = empty_payload();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let w = Link {
            params: LinkParams {
                url: "https://example.com".into(),
                label: Some("Example".into()),
            },
        };
        let out = w.render(&ctx).unwrap();
        // Проверяем точные байты OSC 8: ESC ] 8 ; ; URL ESC \ TEXT ESC ] 8 ; ; ESC \
        assert_eq!(
            out,
            "\u{1b}]8;;https://example.com\u{1b}\\Example\u{1b}]8;;\u{1b}\\"
        );
        // И семантические проверки:
        assert!(out.starts_with("\u{1b}]8;;"));
        assert!(out.contains("Example"));
        assert!(out.ends_with("\u{1b}]8;;\u{1b}\\"));
    }

    #[test]
    fn link_falls_back_to_url_when_label_absent() {
        let p = empty_payload();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let w = Link {
            params: LinkParams {
                url: "https://x.com".into(),
                label: None,
            },
        };
        let out = w.render(&ctx).unwrap();
        // label секция = url
        assert_eq!(
            out,
            "\u{1b}]8;;https://x.com\u{1b}\\https://x.com\u{1b}]8;;\u{1b}\\"
        );
    }

    #[test]
    fn link_returns_none_for_empty_url() {
        let p = empty_payload();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let w = Link {
            params: LinkParams {
                url: String::new(),
                label: Some("ignored".into()),
            },
        };
        assert_eq!(w.render(&ctx), None);
    }
}
```

- [ ] **Step 2: Запустить — должны упасть на компиляции (модуль не подключён)**

```bash
cargo test --locked --lib widgets::static_text 2>&1 | head -10
```

Expected: `error[E0583]: file not found for module ...` или `module ... not found in ...` — потому что `widgets/mod.rs` не объявляет `pub mod static_text;`.

- [ ] **Step 3: Подключить `pub mod static_text;` и заменить 3 stub'а в `build_one`**

Edit `src/widgets/mod.rs`:

Edit 1:
- `old_string`: `pub mod model;`
- `new_string`: `pub mod model;\npub mod static_text;`
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

Edit 2 (заменить 3 Stub-arm на реальные impl):
- `old_string`:
  ```rust
          // Phase 3 stubs — заменяются на реальные impl в T2–T7:
          WidgetConfig::CustomText { .. } => Box::new(Stub("CustomText")),
          WidgetConfig::CustomSymbol { .. } => Box::new(Stub("CustomSymbol")),
          WidgetConfig::Link { .. } => Box::new(Stub("Link")),
  ```
- `new_string`:
  ```rust
          // Phase 3 — Task 2 (static cluster):
          WidgetConfig::CustomText { params } => Box::new(static_text::CustomText {
              params: params.clone(),
          }),
          WidgetConfig::CustomSymbol { params } => Box::new(static_text::CustomSymbol {
              params: params.clone(),
          }),
          WidgetConfig::Link { params } => Box::new(static_text::Link {
              params: params.clone(),
          }),

          // Phase 3 stubs — заменяются на реальные impl в T3–T7:
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

- [ ] **Step 4: Запустить тесты — должны быть зелёные**

```bash
cargo test --locked --lib widgets::static_text
```

Expected: 7 тестов passed:
```
test widgets::static_text::tests::custom_text_renders_literal ... ok
test widgets::static_text::tests::custom_text_returns_none_for_empty ... ok
test widgets::static_text::tests::custom_symbol_renders_literal ... ok
test widgets::static_text::tests::custom_symbol_returns_none_for_empty ... ok
test widgets::static_text::tests::link_emits_osc8_with_label ... ok
test widgets::static_text::tests::link_falls_back_to_url_when_label_absent ... ok
test widgets::static_text::tests::link_returns_none_for_empty_url ... ok

test result: ok. 7 passed
```

- [ ] **Step 5: Smoke-тест end-to-end через `cargo run`**

```bash
echo '{"session_id":"x","model":{"id":"m","display_name":"M"},"workspace":{"current_dir":"/tmp"}}' \
  | RUST_LOG=info cargo run --release -- 2>/dev/null
```

Expected: `M` (default-line содержит только `Model`).

Это просто sanity-check, что Phase 2 рендер не сломан после введения новых stub-вариантов в `build_one`.

- [ ] **Step 6: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0. Тестов суммарно теперь ≥36.

Возможные clippy-замечания:
- `pedantic::missing_errors_doc` на `Widget::render` — игнорируем (`Option<String>` не Result).
- `pedantic::trivially_copy_pass_by_ref` на `&self` в `render` — это требование trait'а, игнорируем.
- `pedantic::needless_pass_by_value` на `params: ContextBarParams` в Stub: не релевантно, `Stub` берёт `&'static str`.

Если clippy ругается на cyclomatic complexity `build_one` после раздутия — добавить `#[allow(clippy::too_many_lines)]` к функции.

- [ ] **Step 7: Verification — task-specific gate**

```bash
grep -c 'pub struct CustomText' src/widgets/static_text.rs
grep -c 'pub struct CustomSymbol' src/widgets/static_text.rs
grep -c 'pub struct Link' src/widgets/static_text.rs
grep -c 'static_text::CustomText' src/widgets/mod.rs
grep -c 'static_text::CustomSymbol' src/widgets/mod.rs
grep -c 'static_text::Link' src/widgets/mod.rs
grep -c '\\x1b\]8;;' src/widgets/static_text.rs
```

Expected output:
```
1
1
1
1
1
1
≥1   (один format! с OSC 8)
```

- [ ] **Step 8: Commit**

```bash
git add src/widgets/static_text.rs src/widgets/mod.rs
git commit -m "feat(phase-3): T2 static cluster — CustomText, CustomSymbol, Link

Static-text widgets — read no payload, render literal config strings.
Link emits OSC 8 hyperlink escape (ESC ] 8 ;; URL ESC \\ TEXT
ESC ] 8 ;; ESC \\); terminal capability detection deferred to Phase 7.
Phase 4 will add color/bold/bg overrides.

Empty url/text/symbol → None (filtered by Plain renderer).

Task 2/9 of Phase 3.
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

- [ ] `src/widgets/static_text.rs` создан, содержит `CustomText`, `CustomSymbol`, `Link`
- [ ] Каждый виджет реализует `Widget` trait с правильным `id()`
- [ ] `Link::render` эмиттит OSC 8 `\x1b]8;;URL\x1b\\TEXT\x1b]8;;\x1b\\`
- [ ] `Link` без `label` использует `url` как label
- [ ] Пустой `url`/`text`/`symbol` → `None`
- [ ] `widgets::mod` подключает `pub mod static_text;` и инстанциирует 3 widget'а в `build_one`
- [ ] 7 unit-тестов в `widgets::static_text::tests` зелёные
- [ ] Phase 2 рендер не регрессирует (snapshot'ы default-line зелёные)
- [ ] Один commit `feat(phase-3): T2 static cluster ...`

## Files touched

- `src/widgets/static_text.rs` (created)
- `src/widgets/mod.rs` (modified)

## Risks & rollback

- **OSC 8 escape-байты ломают Plain-рендер**: `Plain.render()` объединяет сегменты через `" | "`, не парсит escape. Сегмент с `\x1b]8;;...\x1b\\` пройдёт целиком. `Plain.filter(|s| !s.is_empty())` тоже не зацепит (наш сегмент непустой).
- **Snapshot-тест падает на сырых escape-байтах**: snapshot'ы Phase 2 задают только default-line с `Model`; OSC 8 в них не попадёт. Snapshot'ы Phase 3 (Task 8) с явным `Link`-сценарием будут содержать escape-байты — `insta::assert_snapshot!` сериализует их как литералы (`\x1b`), визуально чисто.
- **`params.clone()` в `build_one`**: `CustomTextParams` содержит `String`, clone аллоцирует. Это разовая стоимость на старте процесса, не hot-path. Бюджет 4.5 ms (spec) спокойно покрывает.
- **`label: None` + пустой `url`**: тест `link_returns_none_for_empty_url` лочит поведение.
- **Rollback**: `git revert HEAD` — снимает кластер, `build_one` возвращается к Stub'ам.
