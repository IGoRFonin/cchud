# Task 11 — render rewrite (flex.rs + render_line refactor + Plain/Powerline globals + AlignRight)

**Цель:** Самая большая задача Phase 7 — рефакторинг render-pipeline.
1. Реализовать `render/flex.rs` (`flex_budget` + `truncate_to_budget`).
2. Добавить `Segment.align_marker: bool`.
3. Заменить `Renderer::render(&segments)` на `Renderer::render_line(&segments, &mut state, &theme)`.
4. Plain/Powerline принимают state + theme; реализуют globalBold (через `apply_widget_style` уже applied на этапе main), `override_*_color`, `minimalist_mode`, `inherit_separator_colors` (Powerline only), `auto_align` (Powerline only).
5. `WidgetConfig::AlignRight` → real sentinel widget с `id() == "align-right"`.
6. **Critical guard:** Phase 4 snapshot'ы остаются byte-identical после миграции.

**Files:**
- Modify: `src/render/flex.rs` — `flex_budget`/`truncate_to_budget`
- Modify: `src/render/mod.rs` — `Segment.align_marker`, `Renderer::render_line`, обновить тесты
- Modify: `src/render/plain.rs` — `render_line(segments, state, theme)`, поддерживает minimalist
- Modify: `src/render/powerline.rs` — то же + inherit_separator_colors + auto_align via AlignRight
- Modify: `src/widgets/mod.rs::build_one` — `WidgetConfig::AlignRight` → новый `AlignRightSentinel`
- Modify: `src/main.rs` — pass state + theme, set align_marker
- Modify: `src/util/mod.rs` — добавить `terminal_width()` helper (если ещё нет)

---

- [ ] **Step 1: Реализовать flex.rs**

В `src/render/flex.rs`:

```rust
//! Phase 7 Task 11 — flex_mode width truncation.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::types::config::FlexMode;
use crate::util::ansi::visible_width;

/// Returns truncate budget. None = no truncation.
#[must_use]
pub fn flex_budget(mode: FlexMode, term_width: usize) -> Option<usize> {
    match mode {
        FlexMode::Disabled => None,
        FlexMode::Full => Some(term_width),
        FlexMode::FullMinus20 => term_width.checked_sub(20),
        FlexMode::FullMinus40 => term_width.checked_sub(40),
    }
}

/// Truncate `rendered` to `budget` visible columns. Adds ellipsis `…` if truncated.
#[must_use]
pub fn truncate_to_budget(rendered: &str, budget: Option<usize>) -> String {
    let Some(budget) = budget else { return rendered.to_string(); };
    let visible = visible_width(rendered);
    if visible <= budget {
        return rendered.to_string();
    }
    let target = budget.saturating_sub(1);
    let truncated = truncate_visible(rendered, target);
    format!("{truncated}…")
}

/// Truncate to `target` visible columns, preserving ANSI escape sequences as-is.
fn truncate_visible(s: &str, target: usize) -> String {
    use unicode_segmentation::UnicodeSegmentation;
    use unicode_width::UnicodeWidthStr;

    let mut out = String::new();
    let mut visible = 0_usize;
    let mut in_escape = false;

    for g in s.graphemes(true) {
        if g == "\u{1b}" {
            in_escape = true;
            out.push_str(g);
            continue;
        }
        if in_escape {
            out.push_str(g);
            // ANSI CSI ends on a letter in 0x40..=0x7e; OSC ends on BEL or ST.
            if g.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == 'm' || c == '\\') {
                in_escape = false;
            }
            continue;
        }
        let w = UnicodeWidthStr::width(g);
        if visible + w > target {
            break;
        }
        out.push_str(g);
        visible += w;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_full_returns_term_width() {
        assert_eq!(flex_budget(FlexMode::Full, 80), Some(80));
    }

    #[test]
    fn budget_full_minus_40_subtracts() {
        assert_eq!(flex_budget(FlexMode::FullMinus40, 100), Some(60));
    }

    #[test]
    fn budget_disabled_returns_none() {
        assert_eq!(flex_budget(FlexMode::Disabled, 100), None);
    }

    #[test]
    fn budget_saturating_sub_for_narrow_term() {
        assert_eq!(flex_budget(FlexMode::FullMinus40, 30), None);
    }

    #[test]
    fn truncate_short_returns_unchanged() {
        assert_eq!(truncate_to_budget("hello", Some(80)), "hello");
    }

    #[test]
    fn truncate_long_with_ellipsis() {
        let out = truncate_to_budget("hello world", Some(7));
        assert_eq!(out, "hello …");
    }

    #[test]
    fn truncate_none_budget_returns_unchanged() {
        assert_eq!(truncate_to_budget("any string", None), "any string");
    }
}
```

(Если `crate::util::ansi::visible_width` не существует под этим именем — проверить `src/util/ansi.rs` и адаптировать use.)

- [ ] **Step 2: Run flex tests**

Run: `cargo test --lib render::flex`
Expected: PASS.

- [ ] **Step 3: Добавить `terminal_width` helper в util/mod.rs (если нет)**

```rust
// src/util/mod.rs — добавить если отсутствует
#[must_use]
pub fn terminal_width() -> usize {
    terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .unwrap_or(80)
}
```

- [ ] **Step 4: Добавить `align_marker` в `Segment`**

В `src/render/mod.rs::Segment`:

```rust
#[derive(Debug, Clone)]
pub struct Segment {
    pub text: String,
    pub style: Style,
    pub hyperlink: Option<String>,
    /// Phase 7: marker для `auto_align` — сегмент действует как разделитель left/right.
    pub align_marker: bool,
}

impl Segment {
    #[must_use]
    pub fn plain(text: impl Into<String>) -> Self {
        Self { text: text.into(), style: Style::none(), hyperlink: None, align_marker: false }
    }
    #[must_use]
    pub fn styled(text: impl Into<String>, style: Style) -> Self {
        Self { text: text.into(), style, hyperlink: None, align_marker: false }
    }
}
```

- [ ] **Step 5: Обновить вызовы `Segment {...}` в codebase**

`grep -rn "Segment {" src/ tests/` — обновить literal-формы:

```rust
Segment { text, style, hyperlink, align_marker: false }
```

- [ ] **Step 6: Refactor `Renderer::render` → `render_line(segs, &mut state, theme)`**

В `src/render/mod.rs`:

```rust
impl Renderer {
    pub fn render_line(
        &self,
        segments: &[Segment],
        state: &mut RenderState,
        theme: &crate::types::config::ThemeConfig,
    ) -> String {
        match self {
            Self::Plain(p) => p.render_line(segments, state, theme),
            Self::Powerline(p) => p.render_line(segments, state, theme),
        }
    }
}
```

Удалить старую `Renderer::render(&self, segments)`. Все callers обновить.

- [ ] **Step 7: Plain::render_line с globals**

В `src/render/plain.rs`:

```rust
use super::{ColorLevel, RenderState, Segment, Style};
use crate::types::config::ThemeConfig;

impl Plain {
    #[must_use]
    pub fn render_line(
        &self,
        segments: &[Segment],
        _state: &mut RenderState,
        theme: &ThemeConfig,
    ) -> String {
        let term_width = crate::util::terminal_width();
        let force_minimalist = theme.minimalist_mode
            || (theme.compact_threshold > 0 && term_width < theme.compact_threshold as usize);
        if force_minimalist {
            return render_minimalist(segments);
        }

        let mut parts: Vec<String> = Vec::with_capacity(segments.len());
        for seg in segments {
            if seg.text.is_empty() || seg.align_marker {
                continue;
            }
            // theme globals (override_*_color, global_bold) уже применены в apply_widget_style
            // на этапе сборки сегментов в main.rs.
            let styled = seg.style.render(&seg.text, self.level);
            let with_link = match &seg.hyperlink {
                Some(url) => super::hyperlink::link(&styled, url, self.hyperlinks),
                None => styled,
            };
            parts.push(with_link);
        }
        parts.join(&self.separator)
    }
}

pub(super) fn render_minimalist(segments: &[Segment]) -> String {
    segments
        .iter()
        .filter(|s| !s.text.is_empty() && !s.align_marker)
        .map(|s| strip_emoji_prefix(&s.text))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn strip_emoji_prefix(s: &str) -> String {
    let mut chars = s.chars().peekable();
    while let Some(&c) = chars.peek() {
        let cp = c as u32;
        if c == ' ' || (0x1F000..=0x1FFFF).contains(&cp) {
            chars.next();
        } else {
            break;
        }
    }
    chars.collect()
}
```

- [ ] **Step 8: Powerline::render_line с globals + auto_align + inherit_separator_colors**

В `src/render/powerline.rs::Powerline` заменить `render` на `render_line`:

```rust
use super::{ColorLevel, RenderState, Segment, Style};
use crate::types::config::ThemeConfig;

impl Powerline {
    pub fn render_line(
        &self,
        segments: &[Segment],
        state: &mut RenderState,
        theme: &ThemeConfig,
    ) -> String {
        let term_width = crate::util::terminal_width();
        let force_minimalist = theme.minimalist_mode
            || (theme.compact_threshold > 0 && term_width < theme.compact_threshold as usize);
        if force_minimalist {
            return super::plain::render_minimalist(segments);
        }

        if theme.auto_align {
            if let Some(idx) = segments.iter().position(|s| s.align_marker) {
                let left = self.render_inner(&segments[..idx], state, theme);
                let right = self.render_inner(&segments[idx + 1..], state, theme);
                return pad_to_width(&left, &right, term_width);
            }
        }

        self.render_inner(segments, state, theme)
    }

    fn render_inner(
        &self,
        segments: &[Segment],
        state: &mut RenderState,
        theme: &ThemeConfig,
    ) -> String {
        let visible: Vec<&Segment> = segments
            .iter()
            .filter(|s| !s.text.is_empty() && !s.align_marker)
            .collect();
        if visible.is_empty() { return String::new(); }

        let mut out = String::new();
        let mut prev_bg = self.theme.terminal_bg;

        for seg in visible.iter() {
            let idx = state.global_theme_index;
            let bg = seg.style.bg.unwrap_or_else(|| self.cycle_bg(idx));
            let fg = seg.style.fg.unwrap_or_else(|| self.cycle_fg(idx));

            let (sep_fg, sep_bg) = if theme.inherit_separator_colors {
                (prev_bg, prev_bg)
            } else {
                (prev_bg, bg)
            };

            let sep_style = Style::none().fg(sep_fg).bg(sep_bg);
            out.push_str(&sep_style.render(&self.separator_left.to_string(), self.level));

            let body_text = format!(" {} ", seg.text);
            let body_style = Style {
                fg: Some(fg),
                bg: Some(bg),
                ..seg.style
            };
            let styled_body = body_style.render(&body_text, self.level);
            let body_with_link = match &seg.hyperlink {
                Some(url) => super::hyperlink::link(&styled_body, url, self.hyperlinks),
                None => styled_body,
            };
            out.push_str(&body_with_link);

            prev_bg = bg;
            state.global_theme_index = state.global_theme_index.saturating_add(1);
        }

        let final_sep = Style::none().fg(prev_bg).bg(self.theme.terminal_bg);
        out.push_str(&final_sep.render(&self.separator_left.to_string(), self.level));

        out
    }
}

fn pad_to_width(left: &str, right: &str, width: usize) -> String {
    let lw = crate::util::ansi::visible_width(left);
    let rw = crate::util::ansi::visible_width(right);
    let pad = width.saturating_sub(lw + rw);
    format!("{left}{}{right}", " ".repeat(pad))
}
```

- [ ] **Step 9: build_one для AlignRight**

В `src/widgets/mod.rs` — заменить stub на реальный sentinel:

```rust
        WidgetConfig::AlignRight => Box::new(AlignRightSentinel),
```

И добавить:

```rust
struct AlignRightSentinel;
impl Widget for AlignRightSentinel {
    fn id(&self) -> &'static str { "align-right" }
    fn render(&self, _: &RenderContext<'_>) -> Option<String> { Some(String::new()) }
}
```

- [ ] **Step 10: Update `main.rs` для использования render_line + align_marker**

В `src/main.rs::render_pipeline` обновить блок построения сегментов:

```rust
let widget_items = build_widgets(&settings);
let segments: Vec<crate::render::Segment> = widget_items
    .iter()
    .filter_map(|(w, ovr)| {
        let text = w.render(&ctx)?;
        let is_align = w.id() == "align-right";
        let style = crate::render::apply_widget_style(
            w.default_style(),
            None,  // theme.widget_styles[id] — Phase 4 lookup; пока None
            ovr,
            &settings.theme,
        );
        Some(crate::render::Segment {
            text,
            style,
            hyperlink: w.hyperlink(&ctx),
            align_marker: is_align,
        })
    })
    .collect();

let renderer = Renderer::from_settings(&settings);
let mut state = crate::render::RenderState::default();
println!("{}", renderer.render_line(&segments, &mut state, &settings.theme));
```

(Multi-line — в T12.)

- [ ] **Step 11: Обновить старые render тесты**

Все `r.render(&segs)` → `r.render_line(&segs, &mut RenderState::default(), &ThemeConfig::default())`. Это значит обновить:
- `src/render/mod.rs::renderer_tests` (3 теста)
- `src/render/plain.rs::tests` (5 тестов)
- `src/render/powerline.rs::tests` (5 тестов)

Пример:

```rust
// src/render/plain.rs::tests
fn r(level: ColorLevel, hyperlinks: bool) -> Plain {
    Plain { separator: " | ".into(), level, hyperlinks }
}

#[test]
fn empty_input_yields_empty_string() {
    let mut state = RenderState::default();
    let theme = ThemeConfig::default();
    assert_eq!(r(ColorLevel::None, false).render_line(&[], &mut state, &theme), "");
}
```

- [ ] **Step 12: Run render tests**

Run: `cargo test --lib render::`
Expected: PASS — все unit-тесты Plain/Powerline/RenderState/apply_widget_style/flex.

- [ ] **Step 13: Critical guard — Phase 4 snapshot suite byte-identical**

Run: `cargo test --test snapshots --locked`
Run: `cargo test --test snapshots_transcript --locked`
Run: `cargo test --test snapshots_git --locked`
Expected: PASS — никаких `.snap.new`. Default `RenderState::default()` + `WidgetStyleOverride::default()` + `ThemeConfig::default()` дают identity behavior.

Run: `cargo insta review`
Expected: пусто (нет ожидающих изменений).

**Если есть diff — STOP и расследовать.** Не accept без понимания root cause.

- [ ] **Step 14: Run full test suite + clippy**

```bash
cargo test --locked
cargo clippy --locked -- -D warnings
```

Expected: PASS.

- [ ] **Step 15: Commit**

```bash
git add src/render/{mod,plain,powerline,flex}.rs src/widgets/mod.rs src/main.rs src/util/mod.rs
git commit -m "feat(phase-7): T11 — render_line refactor, flex truncate, AlignRight sentinel, theme globals"
```
