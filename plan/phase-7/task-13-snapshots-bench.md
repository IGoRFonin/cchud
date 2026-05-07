# Task 13 — Snapshots + benchmarks

**Цель:** Создать ≥6 snapshot-фикстур, покрывающих все Phase 7 фичи (usage cluster, env cluster, per-widget overrides, theme globals, multi-line, compact-mode). Создать bench-config `cchud-60w` (все 60 виджетов на двух линиях). Прогнать hyperfine, заполнить targets table.

**Files:**
- Create: `tests/snapshots_phase7.rs` — ≥6 фикстур
- Create: `tests/configs/usage-cluster.json`
- Create: `tests/configs/env-cluster.json`
- Create: `tests/configs/per-widget-override.json`
- Create: `tests/configs/theme-globals.json`
- Create: `tests/configs/multi-line-themed.json`
- Create: `tests/configs/compact-mode.json`
- Create: `benches/configs/phase-7-60w.json`
- Create: `benches/phase-7.md`

---

- [ ] **Step 1: Создать tests/configs/usage-cluster.json**

```json
{
  "version": 1,
  "lines": [{
    "widgets": [
      {"type": "session-usage"},
      {"type": "weekly-usage"},
      {"type": "block-reset-timer"},
      {"type": "weekly-reset-timer"}
    ]
  }],
  "theme": {"kind": "plain"}
}
```

- [ ] **Step 2: Создать tests/configs/per-widget-override.json**

```json
{
  "version": 1,
  "lines": [{
    "widgets": [
      {"type": "model", "color": "#fafafa", "bold": true},
      {"type": "session-cost", "color": "#ff00ff"},
      {"type": "git-branch", "background_color": "#00ff00"}
    ]
  }],
  "theme": {"kind": "powerline", "theme_name": "dracula"}
}
```

- [ ] **Step 3: Создать tests/configs/theme-globals.json**

```json
{
  "version": 1,
  "lines": [{
    "widgets": [
      {"type": "model"}, {"type": "session-cost"}, {"type": "git-branch"}
    ]
  }],
  "theme": {
    "kind": "powerline",
    "theme_name": "default",
    "global_bold": true,
    "inherit_separator_colors": true,
    "override_background_color": "#aabbcc"
  }
}
```

- [ ] **Step 4: Создать tests/configs/multi-line-themed.json**

```json
{
  "version": 1,
  "lines": [
    {"widgets": [
      {"type": "model"}, {"type": "session-cost"},
      {"type": "tokens-total"}, {"type": "context-percentage"}, {"type": "skills"}
    ]},
    {"widgets": [
      {"type": "git-branch"}, {"type": "git-status"},
      {"type": "git-ahead-behind"}, {"type": "git-pr"}, {"type": "free-memory"}
    ]}
  ],
  "theme": {"kind": "powerline", "continue_theme_across_lines": true}
}
```

- [ ] **Step 5: Создать tests/configs/compact-mode.json**

```json
{
  "version": 1,
  "lines": [{
    "widgets": [{"type": "model"}, {"type": "session-cost"}, {"type": "git-branch"}]
  }],
  "theme": {"kind": "powerline", "compact_threshold": 200}
}
```

- [ ] **Step 6: Создать tests/configs/env-cluster.json**

```json
{
  "version": 1,
  "lines": [{
    "widgets": [
      {"type": "claude-account-email"},
      {"type": "free-memory"},
      {"type": "skills"}
    ]
  }],
  "theme": {"kind": "plain"}
}
```

- [ ] **Step 7: Создать tests/snapshots_phase7.rs**

Структура аналогична существующему `tests/snapshots.rs` (Phase 4) и `tests/snapshots_transcript.rs` (Phase 6).

```rust
//! Phase 7 snapshot suite — 6 fixtures covering Phase 7 features.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use cchud::render::{self, RenderState, Renderer, Segment};
use cchud::types::{config::Settings, payload::StatusPayload};
use cchud::widgets::{RenderContext, build_widgets};

fn render_with_fixture(config_path: &str) -> String {
    let cfg_str = std::fs::read_to_string(config_path).unwrap();
    let settings: Settings = serde_json::from_str(&cfg_str).unwrap();
    let payload_str =
        std::fs::read_to_string("benches/samples/payload-cchud-sonnet-xlarge.json").unwrap();
    let payload: StatusPayload = serde_json::from_str(&payload_str).unwrap();
    let mut ctx = RenderContext::new(&payload, &settings);
    ctx.now_ms = 1_745_900_000_000; // зафиксированный момент 2026-04-29 для детерминизма

    let lines = build_widgets(&settings);
    let renderer = Renderer::from_settings(&settings);
    let mut state = RenderState::default();
    let term_width = 120;
    let budget = render::flex::flex_budget(settings.theme.flex_mode, term_width);

    let mut output = String::new();
    for (i, line_widgets) in lines.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        let segments: Vec<Segment> = line_widgets
            .iter()
            .filter_map(|(w, ovr)| {
                let text = w.render(&ctx)?;
                let is_align = w.id() == "align-right";
                let style =
                    render::apply_widget_style(w.default_style(), None, ovr, &settings.theme);
                Some(Segment {
                    text,
                    style,
                    hyperlink: w.hyperlink(&ctx),
                    align_marker: is_align,
                })
            })
            .collect();
        let line_str = renderer.render_line(&segments, &mut state, &settings.theme);
        output.push_str(&render::flex::truncate_to_budget(&line_str, budget));
        if !settings.theme.continue_theme_across_lines {
            state.reset();
        }
    }
    output
}

#[test]
fn usage_cluster() {
    insta::assert_snapshot!(render_with_fixture("tests/configs/usage-cluster.json"));
}

#[test]
fn env_cluster() {
    insta::assert_snapshot!(render_with_fixture("tests/configs/env-cluster.json"));
}

#[test]
fn per_widget_override() {
    insta::assert_snapshot!(render_with_fixture("tests/configs/per-widget-override.json"));
}

#[test]
fn theme_globals() {
    insta::assert_snapshot!(render_with_fixture("tests/configs/theme-globals.json"));
}

#[test]
fn multi_line_themed() {
    insta::assert_snapshot!(render_with_fixture("tests/configs/multi-line-themed.json"));
}

#[test]
fn compact_mode() {
    insta::assert_snapshot!(render_with_fixture("tests/configs/compact-mode.json"));
}
```

NB: убедиться, что `cchud` (lib) reexports `render::apply_widget_style`, `render::RenderState`, `render::Segment`, `render::flex::*`, `render::Renderer`. В `src/lib.rs` добавить недостающие `pub use` или через `pub mod render;`.

- [ ] **Step 8: Снять snapshot baseline**

Run: `cargo insta test --test snapshots_phase7`
Expected: 6 `.snap.new` файлов в `tests/snapshots/`.

Run: `cargo insta accept`
Expected: snapshot'ы committed.

- [ ] **Step 9: Создать benches/configs/phase-7-60w.json**

Полный конфиг со всеми 60 виджетами (multi-line, по 30 на строку):

```json
{
  "version": 1,
  "lines": [
    {"widgets": [
      {"type": "version"}, {"type": "model"}, {"type": "session-name"}, {"type": "session-clock"},
      {"type": "session-cost"}, {"type": "context-length"}, {"type": "context-percentage"},
      {"type": "context-percentage-usable"}, {"type": "context-bar"},
      {"type": "tokens-input"}, {"type": "tokens-output"}, {"type": "tokens-cached"},
      {"type": "tokens-total"}, {"type": "input-speed"}, {"type": "output-speed"}, {"type": "total-speed"},
      {"type": "block-timer"}, {"type": "session-duration"}, {"type": "thinking-effort"}, {"type": "skills"},
      {"type": "session-usage"}, {"type": "weekly-usage"}, {"type": "block-reset-timer"}, {"type": "weekly-reset-timer"},
      {"type": "claude-session-id"}, {"type": "terminal-width"}, {"type": "output-style"}, {"type": "vim-mode"},
      {"type": "claude-account-email"}, {"type": "free-memory"}
    ]},
    {"widgets": [
      {"type": "git-branch"}, {"type": "git-sha"}, {"type": "git-root-dir"},
      {"type": "git-status"}, {"type": "git-changes"}, {"type": "git-staged"},
      {"type": "git-unstaged"}, {"type": "git-untracked"}, {"type": "git-conflicts"},
      {"type": "git-insertions"}, {"type": "git-deletions"}, {"type": "git-ahead-behind"},
      {"type": "git-origin-owner"}, {"type": "git-origin-repo"}, {"type": "git-origin-owner-repo"},
      {"type": "git-upstream-owner"}, {"type": "git-upstream-repo"}, {"type": "git-upstream-owner-repo"},
      {"type": "git-is-fork"}, {"type": "git-pr"},
      {"type": "worktree"}, {"type": "worktree-mode"}, {"type": "worktree-name"},
      {"type": "worktree-branch"}, {"type": "worktree-original-branch"},
      {"type": "custom-text", "text": "[]"}, {"type": "custom-symbol", "symbol": "★"},
      {"type": "link", "url": "https://x.com", "label": "X"}, {"type": "context-bar", "width": 20},
      {"type": "custom-command", "command": "echo", "args": ["hi"], "timeout_ms": 50}
    ]}
  ],
  "theme": {"kind": "powerline", "theme_name": "dracula"}
}
```

- [ ] **Step 10: Создать benches/phase-7.md с hyperfine targets**

```markdown
# Phase 7 Benchmarks

## Targets
- cchud-8w: < 5 ms p95
- cchud-60w: < 12 ms p95
- cchud-20w (Phase 6 baseline): без регрессии > 10%
- Cold parse 50 MB transcript: < 11 ms (было 10 ms; +1 ms на skills)
- Warm + 1 MB append: < 3.5 ms (было 3 ms; +0.5 ms на skills merge)

## Run

\```bash
PAYLOAD=$(cat benches/samples/payload-cchud-sonnet-xlarge.json)

for cfg in 8w 20w 60w; do
  hyperfine --warmup 3 \
    "echo '$PAYLOAD' | CCHUD_CONFIG=benches/configs/phase-{6,7}-${cfg}.json ./target/release/cchud" \
    --export-json benches/results/phase-7-${cfg}.json
done
\```

## Results

(Заполнить после первого прогона на ноуте автора.)

| Config | mean | p95 | regression vs Phase 6 |
|---|---|---|---|
| cchud-8w | TBD | TBD | TBD |
| cchud-20w | TBD | TBD | TBD |
| cchud-60w | TBD | TBD | TBD |
| cold parse 50 MB | TBD | TBD | TBD |
| warm + 1 MB | TBD | TBD | TBD |
```

(После прогона T13 заполнить TBD реальными числами.)

- [ ] **Step 11: Run hyperfine benches**

```bash
cargo build --release --locked
PAYLOAD=$(cat benches/samples/payload-cchud-sonnet-xlarge.json)
for cfg in phase-6-8w phase-6-20w phase-7-60w; do
  hyperfine --warmup 3 \
    "echo '$PAYLOAD' | CCHUD_CONFIG=benches/configs/${cfg}.json ./target/release/cchud" \
    --export-json benches/results/${cfg}.json || true
done
```

Expected: targets met. Заполнить таблицу в `benches/phase-7.md`.

(Если для `cchud-8w`/`cchud-20w` нет соответствующих configs — переиспользовать Phase 5/6 configs `benches/configs/phase-{5,6}-*.json` или создать новые `benches/configs/phase-7-{8,20}w.json` подмножествами 60w.)

- [ ] **Step 12: Run all snapshot suites**

```bash
cargo test --test snapshots --locked
cargo test --test snapshots_git --locked
cargo test --test snapshots_transcript --locked
cargo test --test snapshots_phase7 --locked
```

Expected: PASS — все 4 suite зелёные, ноль `.snap.new`.

- [ ] **Step 13: Run full test suite + clippy + fmt**

```bash
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: всё PASS.

- [ ] **Step 14: Commit**

```bash
git add tests/configs/*.json tests/snapshots_phase7.rs tests/snapshots/snapshots_phase7__*.snap \
        benches/configs/phase-7-*.json benches/phase-7.md benches/results/phase-7-*.json
git commit -m "test(phase-7): T13 — 6 snapshots + hyperfine benches (8w<5ms / 60w<12ms)"
```
