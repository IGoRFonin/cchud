# Task 2 — `compose_line` refactor (STOP-gate snapshot regression)

**Цель:** Extract `Renderer::compose_line` (pure → `Vec<StyledSegment>`) и общий `Renderer::emit_ansi(&[StyledSegment]) -> String`. После refactor TUI live preview переиспользует ту же composition-логику через `style_map::to_span` (Task 3 / Task 9). **Это самая рискованная задача Phase 8** — Phase 4 + Phase 7 snapshots должны остаться byte-identical.

**Files:**
- Modify: `src/render/mod.rs:50-275` — добавить `StyledSegment`, `compose_line`, `emit_ansi`, `for_preview`; `render_line` = `compose_line + emit_ansi`
- Modify: `src/render/plain.rs:14-58` — extract `compose_inner` → `Vec<StyledSegment>`; `render_line` дёргает `compose_inner` + `emit_ansi`
- Modify: `src/render/powerline.rs:46-120` — extract `compose_inner` → `Vec<StyledSegment>`; same pattern
- Modify: `src/render/hyperlink.rs` — добавить `pub fn wrap(text: &str, url: &str) -> String` (если ещё нет — проверить; иначе keep `link()`)
- Test: `src/render/mod.rs::compose_line_tests` — ≥3 unit-теста на новую функцию
- Test: `tests/snapshots.rs` (Phase 4) — **byte-identical** (STOP-gate)
- Test: `tests/snapshots_phase7.rs` (Phase 7) — **byte-identical** (STOP-gate)
- Test: `tests/snapshots_git.rs` + `tests/snapshots_transcript.rs` — **byte-identical**

---

- [ ] **Step 1: Pre-flight baseline**

Зафиксируй baseline-snapshot. Если до этой задачи запустить `cargo test --test snapshots --test snapshots_phase7 --test snapshots_git --test snapshots_transcript`, всё должно быть зелёным. Если нет — STOP, не приступать к refactor.

```bash
cargo test --locked --test snapshots --test snapshots_phase7 --test snapshots_git --test snapshots_transcript
```

Expected: PASS. Запиши количество тестов в каждом файле — не должно измениться после refactor.

- [ ] **Step 2: Добавить `StyledSegment` в `src/render/mod.rs`**

После определения `Segment` (строка 393) добавить:

```rust
/// Сегмент после применения flex truncate, separators (Plain ` | ` или Powerline
/// chevrons), auto_align padding и `apply_widget_style`. Готов к рендерингу через
/// ANSI escape (`Renderer::emit_ansi`) или ratatui Span (`tui::style_map::to_span`).
///
/// Phase 8 Task 2 — единая composition-точка между hot-path и TUI preview.
#[derive(Debug, Clone)]
pub struct StyledSegment {
    pub text: String,
    pub style: Style,
    pub hyperlink: Option<String>,
}

impl StyledSegment {
    #[must_use]
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::none(),
            hyperlink: None,
        }
    }

    #[must_use]
    pub fn styled(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
            hyperlink: None,
        }
    }
}
```

- [ ] **Step 3: Extract `Plain::compose_inner` (pure)**

В `src/render/plain.rs` добавить новый метод; `render_line` начнёт его звать. `compose_inner` отдаёт `Vec<StyledSegment>`, отвечает за фильтрацию empty/align_marker, минималист-fallback, separator-вставку.

В `src/render/plain.rs` заменить блок `impl Plain { ... pub fn render_line ... }` на:

```rust
impl Plain {
    /// Pure: composes raw `Segment`'ы в финальную последовательность `StyledSegment`'ов.
    /// Plain-ветка вставляет separator-сегменты `" | "` со `Style::none()` между виджетами,
    /// фильтрует empty/align_marker, делает auto_align split (через AlignRight sentinel).
    /// Минималист-fallback (`minimalist_mode` или `compact_threshold > term_width`) делает
    /// собственный composition (без separator-цвета).
    #[must_use]
    pub fn compose_inner(
        &self,
        segments: &[super::Segment],
        _state: &mut super::RenderState,
        theme: &ThemeConfig,
    ) -> Vec<super::StyledSegment> {
        let term_width = crate::util::terminal_width();
        let force_minimalist = theme.minimalist_mode
            || (theme.compact_threshold > 0 && term_width < theme.compact_threshold as usize);
        if force_minimalist {
            return compose_minimalist(segments);
        }

        let visible: Vec<&super::Segment> = segments
            .iter()
            .filter(|s| !s.text.is_empty() && !s.align_marker)
            .collect();

        let mut out = Vec::with_capacity(visible.len() * 2);
        for (i, seg) in visible.iter().enumerate() {
            if i > 0 {
                out.push(super::StyledSegment::plain(self.separator.clone()));
            }
            out.push(super::StyledSegment {
                text: seg.text.clone(),
                style: seg.style,
                hyperlink: seg.hyperlink.clone(),
            });
        }
        out
    }

    #[must_use]
    pub fn render_line(
        &self,
        segments: &[super::Segment],
        state: &mut super::RenderState,
        theme: &ThemeConfig,
    ) -> String {
        let composed = self.compose_inner(segments, state, theme);
        emit_plain(&composed, self.level, self.hyperlinks)
    }
}

/// Helper для plain emit. ANSI emit + OSC 8 wrap. Используется только из
/// `Plain::render_line` — TUI берёт composed segments и не вызывает этот path.
fn emit_plain(composed: &[super::StyledSegment], level: super::ColorLevel, hyperlinks: bool) -> String {
    composed
        .iter()
        .map(|s| {
            let painted = s.style.render(&s.text, level);
            match &s.hyperlink {
                Some(url) => link(&painted, url, hyperlinks),
                None => painted,
            }
        })
        .collect::<String>()
}

/// Минималист-режим composing. Аналог старого `render_minimalist`, но возвращает
/// `Vec<StyledSegment>` (без стиля). `Powerline` тоже использует через `Plain::compose_minimalist`.
pub(super) fn compose_minimalist(segments: &[super::Segment]) -> Vec<super::StyledSegment> {
    let visible: Vec<&super::Segment> = segments
        .iter()
        .filter(|s| !s.text.is_empty() && !s.align_marker)
        .collect();
    if visible.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(visible.len() * 2);
    for (i, seg) in visible.iter().enumerate() {
        if i > 0 {
            out.push(super::StyledSegment::plain(" | "));
        }
        out.push(super::StyledSegment::plain(strip_emoji_prefix(&seg.text)));
    }
    out
}
```

Старый `pub(super) fn render_minimalist(...) -> String` оставить (его дёргает Powerline `auto_align` minimalist fallback) — но переписать через `compose_minimalist` + concat:

```rust
pub(super) fn render_minimalist(segments: &[super::Segment]) -> String {
    compose_minimalist(segments)
        .iter()
        .map(|s| s.text.clone())
        .collect::<String>()
}
```

(Так старая plain-tests не сломаются. Удаляем legacy после T13.)

- [ ] **Step 4: Run plain tests**

```bash
cargo test --locked render::plain::tests
```

Expected: PASS. Все 7 тестов выживают (filters_empty, level_none, truecolor, hyperlink_wraps, hyperlink_dropped, align_marker_skipped, empty_input).

- [ ] **Step 5: Extract `Powerline::compose_inner` (pure)**

В `src/render/powerline.rs` сейчас render-логика разделена между `render_line` (auto_align split + minimalist fallback) и `render_inner` (chevron + cycle + body). Powerline самый сложный — chevron-сегменты должны быть полноценными `StyledSegment`'ами в composed.

Заменить `render_inner` целиком, ввести новый `compose_inner` (pure compose без emit), `render_line` использует `compose_inner + emit_powerline`:

```rust
impl Powerline {
    #[must_use]
    pub fn render_line(
        &self,
        segments: &[super::Segment],
        state: &mut super::RenderState,
        theme: &ThemeConfig,
    ) -> String {
        let composed = self.compose_inner(segments, state, theme);
        emit_powerline(&composed, self.level, self.hyperlinks)
    }

    /// Pure: composes Powerline-line в `Vec<StyledSegment>`. Делает chevron-вставку
    /// (`StyledSegment` со стилем `(fg = prev_bg, bg = current_bg)`), body-сегмент с
    /// applied apply_widget_style, финальный transition-chevron в `terminal_bg`.
    /// Auto_align — split на left/right через AlignRight sentinel + padding-сегмент.
    /// Минималист-fallback делегирует в `Plain::compose_minimalist`.
    #[must_use]
    pub fn compose_inner(
        &self,
        segments: &[super::Segment],
        state: &mut super::RenderState,
        theme: &ThemeConfig,
    ) -> Vec<super::StyledSegment> {
        let term_width = crate::util::terminal_width();
        let force_minimalist = theme.minimalist_mode
            || (theme.compact_threshold > 0 && term_width < theme.compact_threshold as usize);
        if force_minimalist {
            return super::plain::compose_minimalist(segments);
        }

        if theme.auto_align {
            if let Some(idx) = segments.iter().position(|s| s.align_marker) {
                let left = self.compose_segments(&segments[..idx], state, theme);
                let right = self.compose_segments(&segments[idx + 1..], state, theme);
                return pad_with_segment(left, right, term_width);
            }
        }
        self.compose_segments(segments, state, theme)
    }

    fn compose_segments(
        &self,
        segments: &[super::Segment],
        state: &mut super::RenderState,
        theme: &ThemeConfig,
    ) -> Vec<super::StyledSegment> {
        let visible: Vec<&super::Segment> = segments
            .iter()
            .filter(|s| !s.text.is_empty() && !s.align_marker)
            .collect();
        if visible.is_empty() {
            return Vec::new();
        }

        let mut out = Vec::with_capacity(visible.len() * 2 + 1);
        let mut prev_bg = self.theme.terminal_bg;

        for seg in &visible {
            let idx = state.global_theme_index;
            let bg = seg.style.bg.unwrap_or_else(|| self.cycle_bg(idx));
            let fg = seg.style.fg.unwrap_or_else(|| self.cycle_fg(idx));

            let sep_style = if theme.inherit_separator_colors {
                super::Style::none().fg(prev_bg).bg(prev_bg)
            } else {
                super::Style::none().fg(prev_bg).bg(bg)
            };
            out.push(super::StyledSegment {
                text: self.separator_left.to_string(),
                style: sep_style,
                hyperlink: None,
            });

            let body_style = super::Style {
                fg: Some(fg),
                bg: Some(bg),
                ..seg.style
            };
            out.push(super::StyledSegment {
                text: format!(" {} ", seg.text),
                style: body_style,
                hyperlink: seg.hyperlink.clone(),
            });

            prev_bg = bg;
            state.global_theme_index = state.global_theme_index.saturating_add(1);
        }

        out.push(super::StyledSegment {
            text: self.separator_left.to_string(),
            style: super::Style::none().fg(prev_bg).bg(self.theme.terminal_bg),
            hyperlink: None,
        });
        out
    }

    fn cycle_bg(&self, idx: usize) -> super::Color {
        let cycle = &self.theme.bg_cycle;
        if cycle.is_empty() {
            self.theme.default_bg
        } else {
            cycle[idx % cycle.len()]
        }
    }

    fn cycle_fg(&self, idx: usize) -> super::Color {
        let cycle = &self.theme.fg_cycle;
        if cycle.is_empty() {
            self.theme.default_fg
        } else {
            cycle[idx % cycle.len()]
        }
    }
}

fn emit_powerline(composed: &[super::StyledSegment], level: super::ColorLevel, hyperlinks: bool) -> String {
    composed
        .iter()
        .map(|s| {
            let painted = s.style.render(&s.text, level);
            match &s.hyperlink {
                Some(url) => link(&painted, url, hyperlinks),
                None => painted,
            }
        })
        .collect::<String>()
}

/// Слепляет left + padding-сегмент + right; padding — visible_width-aware.
/// Возвращает Vec<StyledSegment>; padding-сегмент со `Style::none()`.
fn pad_with_segment(
    mut left: Vec<super::StyledSegment>,
    mut right: Vec<super::StyledSegment>,
    width: usize,
) -> Vec<super::StyledSegment> {
    let lw: usize = left
        .iter()
        .map(|s| crate::util::ansi::visible_width(&s.text))
        .sum();
    let rw: usize = right
        .iter()
        .map(|s| crate::util::ansi::visible_width(&s.text))
        .sum();
    let pad = width.saturating_sub(lw + rw);
    if pad > 0 {
        left.push(super::StyledSegment::plain(" ".repeat(pad)));
    }
    left.append(&mut right);
    left
}
```

Удалить старый `pad_to_width` (он работал на &str ANSI; теперь padding — segment). Удалить `render_inner`.

- [ ] **Step 6: Run powerline tests**

```bash
cargo test --locked render::powerline::tests
```

Expected: PASS. Все 12 тестов выживают.

- [ ] **Step 7: Add `Renderer::compose_line` + `Renderer::emit_ansi` + `for_preview` в `src/render/mod.rs`**

В `impl Renderer` блок (строка 217), после `from_settings`, заменить `render_line` на:

```rust
impl Renderer {
    #[must_use]
    pub fn from_settings(settings: &Settings) -> Self {
        // ... (existing body, keep as-is)
    }

    /// Pure (без IO). Composes raw segments в финальную последовательность
    /// `StyledSegment`'ов через Plain или Powerline ветку (диспатч по `theme.kind`).
    ///
    /// TUI live preview вызывает этот метод и затем мапит каждый `StyledSegment`
    /// на `ratatui::text::Span` через `tui::style_map::to_span`. Hot path
    /// (`Renderer::render_line`) вызывает + `emit_ansi`.
    #[must_use]
    pub fn compose_line(
        &self,
        segments: &[Segment],
        state: &mut RenderState,
        theme: &crate::types::config::ThemeConfig,
    ) -> Vec<StyledSegment> {
        match self {
            Self::Plain(p) => p.compose_inner(segments, state, theme),
            Self::Powerline(p) => p.compose_inner(segments, state, theme),
        }
    }

    /// Тривиальный ANSI emit над composed segments. Plain и Powerline после
    /// refactor имеют общий emit-path: каждый StyledSegment уже содержит финальный
    /// `Style`, остаётся только `Style::render(&text, level)` + опционально OSC 8 wrap.
    #[must_use]
    pub fn emit_ansi(&self, composed: &[StyledSegment]) -> String {
        let (level, hyperlinks) = match self {
            Self::Plain(p) => (p.level, p.hyperlinks),
            Self::Powerline(p) => (p.level, p.hyperlinks),
        };
        composed
            .iter()
            .map(|s| {
                let painted = s.style.render(&s.text, level);
                match &s.hyperlink {
                    Some(url) => hyperlink::link(&painted, url, hyperlinks),
                    None => painted,
                }
            })
            .collect::<String>()
    }

    /// Backward-compatible API. Hot path (main.rs / snapshots / tests) дёргает
    /// именно этот метод. Поведение байт-эквивалентно Phase 7.
    #[must_use]
    pub fn render_line(
        &self,
        segments: &[Segment],
        state: &mut RenderState,
        theme: &crate::types::config::ThemeConfig,
    ) -> String {
        let composed = self.compose_line(segments, state, theme);
        self.emit_ansi(&composed)
    }

    /// TUI-only constructor. Forces `ColorLevel::TrueColor` (preview всегда богатый цвет)
    /// + `hyperlinks = false` (TUI не рендерит OSC 8 — ratatui не понимает escape).
    /// Использует `Settings::theme.kind` для диспатча Plain/Powerline.
    #[cfg(feature = "tui")]
    #[must_use]
    pub fn for_preview(settings: &Settings) -> Self {
        use crate::types::config::ThemeKind;

        match settings.theme.kind {
            ThemeKind::Plain => Self::Plain(plain::Plain {
                separator: " | ".into(),
                level: ColorLevel::TrueColor,
                hyperlinks: false,
            }),
            ThemeKind::Powerline => {
                let theme: themes::PowerlineTheme = settings
                    .theme
                    .custom
                    .clone()
                    .or_else(|| {
                        settings
                            .theme
                            .theme_name
                            .as_deref()
                            .and_then(themes::lookup)
                            .map(Into::into)
                    })
                    .unwrap_or_else(|| (&themes::DEFAULT).into());
                let mut p = powerline::Powerline::new(theme, ColorLevel::TrueColor, false);
                if let Some(sep) = settings
                    .theme
                    .separators
                    .first()
                    .and_then(|s| s.chars().next())
                {
                    p.separator_left = sep;
                }
                Self::Powerline(p)
            }
        }
    }
}
```

- [ ] **Step 8: ⚠️ STOP-gate — Phase 4 + Phase 7 snapshots byte-identical**

Run baseline tests:

```bash
cargo test --locked --test snapshots
cargo test --locked --test snapshots_phase7
cargo test --locked --test snapshots_git
cargo test --locked --test snapshots_transcript
```

**Expected: ALL PASS**, никакого `cargo insta review` не требуется.

Если snapshot diff обнаружен:
1. `cargo insta diff` — посмотреть что изменилось.
2. **Не принимать diff автоматически.** Проблема в `compose_inner` (Plain или Powerline). Скорее всего — порядок separator/body, или прочерк finals_sep, или auto_align padding.
3. Фиксить `compose_inner` пока snapshots не станут byte-identical.
4. **Только после зелёных snapshots** двигаться к Step 9.

Если diff в `tests/snapshots_phase7.rs` (Phase 7 specifics: `theme_globals`, `auto_align`, `multi_line_themed`, `compact_mode`) — особое внимание `auto_align` (Powerline split) и `compact_mode` (минималист).

- [ ] **Step 9: Add `compose_line` unit-тесты в `src/render/mod.rs`**

В конце файла (после `#[cfg(test)] mod tests`) добавить:

```rust
#[cfg(test)]
mod compose_line_tests {
    use super::*;
    use crate::types::config::{Settings, ThemeConfig, ThemeKind};

    #[test]
    fn plain_compose_returns_separator_segments_between_widgets() {
        let s = Settings::default();
        let r = Renderer::from_settings(&s);
        let segs = [Segment::plain("a"), Segment::plain("b")];
        let mut state = RenderState::default();
        let composed = r.compose_line(&segs, &mut state, &ThemeConfig::default());
        // Plain: a + " | " + b → 3 segments.
        assert_eq!(composed.len(), 3);
        assert_eq!(composed[0].text, "a");
        assert_eq!(composed[1].text, " | ");
        assert_eq!(composed[2].text, "b");
    }

    #[test]
    fn render_line_equals_emit_of_compose() {
        let s = Settings::default();
        let r = Renderer::from_settings(&s);
        let segs = [Segment::plain("x"), Segment::plain("y")];
        let mut s1 = RenderState::default();
        let mut s2 = RenderState::default();
        let line = r.render_line(&segs, &mut s1, &ThemeConfig::default());
        let composed = r.compose_line(&segs, &mut s2, &ThemeConfig::default());
        let emit = r.emit_ansi(&composed);
        assert_eq!(line, emit, "render_line must equal emit_ansi(compose_line(_))");
    }

    #[test]
    fn powerline_compose_emits_chevrons_with_terminal_bg_finalizer() {
        let json = r#"{"theme": {"kind": "powerline", "theme_name": "dracula"}}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        let r = Renderer::from_settings(&s);
        let segs = [Segment::plain("hi")];
        let mut state = RenderState::default();
        let composed = r.compose_line(&segs, &mut state, &s.theme);
        // Powerline: chevron + body + final-chevron → 3 segments.
        assert_eq!(composed.len(), 3);
        // Last segment — финальный transition в terminal_bg.
        assert!(composed[2].style.bg.is_some());
    }

    #[cfg(feature = "tui")]
    #[test]
    fn for_preview_forces_truecolor_and_disables_hyperlinks() {
        let s = Settings::default();
        let r = Renderer::for_preview(&s);
        match r {
            Renderer::Plain(p) => {
                assert_eq!(p.level, ColorLevel::TrueColor);
                assert!(!p.hyperlinks);
            }
            Renderer::Powerline(_) => panic!("default should be Plain"),
        }
    }
}
```

Run: `cargo test --locked render::compose_line_tests`
Expected: 4 PASS (3 always + 1 cfg-tui).

- [ ] **Step 10: Final verify — все snapshot тесты + lints**

```bash
cargo test --locked --test snapshots --test snapshots_phase7 --test snapshots_git --test snapshots_transcript
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked --no-default-features
```

Expected: всё PASS.

- [ ] **Step 11: Commit**

```bash
git add src/render/
git commit -m "$(cat <<'EOF'
refactor(phase-8): T2 compose_line — extract pure composition; Plain+Powerline emit_ansi unified

- Introduce StyledSegment { text, style, hyperlink } in src/render/mod.rs
- Renderer::compose_line(&[Segment], &mut RenderState, &ThemeConfig) -> Vec<StyledSegment>
- Renderer::emit_ansi(&[StyledSegment]) -> String  (single ANSI emitter for both branches)
- Renderer::render_line = compose_line + emit_ansi (backward compatible)
- Renderer::for_preview(&Settings) -> Self  (cfg(feature="tui"); forces TrueColor + no hyperlinks)
- Plain::compose_inner / Powerline::compose_inner own separators (chevrons, " | ", padding)
- Phase 4 + Phase 7 + git + transcript snapshots: byte-identical (verified)

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```
