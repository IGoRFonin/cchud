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
    insta::assert_snapshot!(render_with_fixture(
        "tests/configs/per-widget-override.json"
    ));
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
