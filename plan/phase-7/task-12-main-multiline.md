# Task 12 — main.rs multi-line orchestration

**Цель:** Расширить `build_widgets` чтобы возвращал `Vec<Vec<...>>` (one inner Vec per line). В `main.rs::render_pipeline` итерировать по `settings.lines`, склеивать через `\n`, применять `flex_budget` truncate, контролировать `RenderState` reset/continue в зависимости от `theme.continue_theme_across_lines`.

**Files:**
- Modify: `src/widgets/mod.rs::build_widgets` — `Vec<Vec<(Widget, Override)>>` (one per line)
- Modify: `src/main.rs::render_pipeline` — multi-line loop

---

- [ ] **Step 1: Update `build_widgets` to handle all lines**

В `src/widgets/mod.rs`:

```rust
#[must_use]
pub fn build_widgets(settings: &Settings) -> Vec<Vec<(Box<dyn Widget>, WidgetStyleOverride)>> {
    settings
        .lines
        .iter()
        .map(|line| {
            line.widgets
                .iter()
                .map(|item| (build_one(&item.kind), item.style.clone()))
                .collect()
        })
        .collect()
}
```

- [ ] **Step 2: Write smoke test для multi-line build_widgets**

Опционально, в `src/widgets/mod.rs`:

```rust
#[cfg(test)]
mod build_widgets_tests {
    use super::*;
    use crate::types::config::Settings;

    #[test]
    fn build_widgets_returns_one_inner_vec_per_line() {
        let json = r#"{
            "lines": [
                {"widgets": [{"type": "model"}]},
                {"widgets": [{"type": "git-branch"}, {"type": "git-status"}]}
            ]
        }"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        let lines = build_widgets(&s);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].len(), 1);
        assert_eq!(lines[1].len(), 2);
    }

    #[test]
    fn build_widgets_empty_settings_yields_empty_vec() {
        let lines = build_widgets(&Settings::default());
        assert!(lines.is_empty());
    }
}
```

- [ ] **Step 3: Run build_widgets tests**

Run: `cargo test --lib widgets::build_widgets_tests`
Expected: PASS.

- [ ] **Step 4: Update main.rs::render_pipeline to multi-line**

В `src/main.rs`:

```rust
fn render_pipeline() -> ExitCode {
    let payload: StatusPayload = match serde_json::from_reader(std::io::stdin().lock()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("cchud: invalid payload: {e}");
            return ExitCode::SUCCESS;
        }
    };
    let settings = config::load();
    let ctx = RenderContext::new(&payload, &settings);
    let lines = build_widgets(&settings);
    let renderer = Renderer::from_settings(&settings);
    let mut state = crate::render::RenderState::default();
    let term_width = crate::util::terminal_width();
    let budget = crate::render::flex::flex_budget(settings.theme.flex_mode, term_width);

    let mut output = String::new();
    for (i, line_widgets) in lines.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        let segments: Vec<crate::render::Segment> = line_widgets
            .iter()
            .filter_map(|(w, ovr)| {
                let text = w.render(&ctx)?;
                let is_align = w.id() == "align-right";
                let style = crate::render::apply_widget_style(
                    w.default_style(),
                    None,
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
        let line_str = renderer.render_line(&segments, &mut state, &settings.theme);
        let truncated = crate::render::flex::truncate_to_budget(&line_str, budget);
        output.push_str(&truncated);
        if !settings.theme.continue_theme_across_lines {
            state.reset();
        }
    }
    println!("{output}");
    ExitCode::SUCCESS
}
```

- [ ] **Step 5: Smoke test multi-line manually**

Создать `/tmp/cchud-multi.json`:

```bash
cat <<'EOF' > /tmp/cchud-multi.json
{"version": 1, "lines": [
  {"widgets": [{"type": "model"}, {"type": "session-cost"}]},
  {"widgets": [{"type": "git-branch"}, {"type": "git-status"}]}
], "theme": {"continue_theme_across_lines": false}}
EOF
```

И test payload:

```bash
echo '{"session_id":"x","model":{"id":"claude-sonnet-4-6","display_name":"Sonnet 4.6"},"workspace":{"current_dir":"/tmp"}}' \
  | CCHUD_CONFIG=/tmp/cchud-multi.json cargo run --release --quiet
```

Expected: вывод содержит `\n` между двумя строками.

- [ ] **Step 6: Run full test suite**

Run: `cargo test --locked`
Expected: PASS — Phase 4 snapshot'ы по-прежнему byte-identical (single-line config → loop с одной итерацией).

- [ ] **Step 7: Commit**

```bash
git add src/main.rs src/widgets/mod.rs
git commit -m "feat(phase-7): T12 — multi-line orchestration in main.rs + flex truncate"
```
