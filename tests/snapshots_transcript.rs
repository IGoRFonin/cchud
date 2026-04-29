//! Transcript snapshot suite — Phase 6 Task 9.
//!
//! `insta::glob` пробегает по `benches/samples/transcripts/*.jsonl` и для
//! каждого файла рендерит 8-widget config. Любые изменения формата вывода
//! зафиксируются как pending snapshot через `cargo insta review`.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use cchud::types::config::{Line, Settings, WidgetConfig};
use cchud::types::payload::{ModelInfo, StatusPayload, Workspace};
use cchud::widgets::{RenderContext, build_widgets};
use std::path::Path;

fn settings_8w() -> Settings {
    Settings {
        version: 1,
        lines: vec![Line {
            widgets: vec![
                WidgetConfig::TokensCached,
                WidgetConfig::TokensTotal,
                WidgetConfig::InputSpeed,
                WidgetConfig::OutputSpeed,
                WidgetConfig::TotalSpeed,
                WidgetConfig::BlockTimer,
                WidgetConfig::SessionDuration,
                WidgetConfig::ThinkingEffort,
            ],
        }],
        theme: Default::default(),
    }
}

fn payload_for(transcript_path: &str) -> StatusPayload {
    StatusPayload {
        session_id: "snapshot".into(),
        model: ModelInfo {
            id: "claude-sonnet-4-6".into(),
            display_name: "Sonnet 4.6".into(),
        },
        workspace: Workspace {
            current_dir: "/tmp".into(),
            project_dir: None,
            added_dirs: None,
        },
        transcript_path: Some(transcript_path.into()),
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

fn render_for(path: &Path) -> String {
    let settings = settings_8w();
    let payload = payload_for(path.to_str().unwrap());
    let mut ctx = RenderContext::new(&payload, &settings);
    // Фиксируем now_ms = 2026-01-01T03:00:00Z чтобы BlockTimer был детерминирован.
    ctx.now_ms = 1_767_236_400_000;
    let widgets = build_widgets(&settings);
    let parts: Vec<String> = widgets
        .iter()
        .map(|w| w.render(&ctx).unwrap_or_else(|| "<none>".into()))
        .collect();
    parts.join(" | ")
}

#[test]
fn transcript_snapshots() {
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("benches/samples/transcripts");
    let mut paths: Vec<_> = std::fs::read_dir(&fixtures_dir)
        .expect("fixtures dir exists")
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            p.extension().map_or(false, |ext| ext == "jsonl") && !name.starts_with("large-")
        })
        .map(|e| e.path())
        .collect();
    paths.sort();

    for path in paths {
        // Очистка кэша перед рендером — каждая итерация стартует cold.
        if let Some(cache) = dirs::cache_dir() {
            let _ = std::fs::remove_dir_all(cache.join("cchud"));
        }
        let rendered = render_for(&path);
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        insta::with_settings!({ snapshot_suffix => name.as_str() }, {
            insta::assert_snapshot!(rendered);
        });
    }
}
