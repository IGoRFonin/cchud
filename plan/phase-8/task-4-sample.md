# Task 4 — `tui::sample` (sample payload + transcript fixture)

**Цель:** Захардкоженный inline `StatusPayload` для live preview + 5-строчный JSONL транскрипт через `tempfile::NamedTempFile`. Tempfile owned `App` через `Option<NamedTempFile>` (RAII). `transcript_path` указывает на путь tempfile.

**Files:**
- Modify: `src/tui/sample.rs` — наполнить (был skeleton после T1)

---

- [ ] **Step 1: Реализовать `payload()` + JSONL fixture**

В `src/tui/sample.rs`:

```rust
//! Sample payload + transcript fixture — Phase 8 Task 4.
//!
//! Inline mock-данные для TUI live preview. Tempfile RAII через NamedTempFile —
//! owned `App` (Task 6) держит его до выхода TUI.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::io::Write;

use tempfile::NamedTempFile;

use crate::types::payload::{
    ContextWindowInfo, CostInfo, CurrentUsage, ModelInfo, OutputStyle, RateBucket, RateLimits,
    StatusPayload, VimState, Workspace, Worktree,
};

/// 5-строчный JSONL транскрипт; покрывает tokens/timing/thinking/skills.
/// Timestamps относительные к now — но детерминированный фикс-набор offset'ов
/// (preview не обязан показывать «свежие» миллисекунды).
const TRANSCRIPT_JSONL: &str = "\
{\"type\":\"user\",\"timestamp\":\"2026-04-29T12:00:00.000Z\",\"message\":{\"role\":\"user\",\"content\":[{\"type\":\"text\",\"text\":\"hello\"}]}}
{\"type\":\"assistant\",\"timestamp\":\"2026-04-29T12:00:01.500Z\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"text\",\"text\":\"hi there\"}],\"usage\":{\"input_tokens\":12,\"output_tokens\":8,\"cache_read_input_tokens\":4096,\"cache_creation_input_tokens\":1024}}}
{\"type\":\"user\",\"timestamp\":\"2026-04-29T12:00:05.000Z\",\"message\":{\"role\":\"user\",\"content\":[{\"type\":\"tool_use\",\"name\":\"Skill\",\"input\":{\"skill\":\"using-superpowers\"}}]}}
{\"type\":\"assistant\",\"timestamp\":\"2026-04-29T12:00:07.250Z\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"thinking\",\"thinking\":\"Reasoning briefly...\"},{\"type\":\"text\",\"text\":\"done\"}],\"usage\":{\"input_tokens\":256,\"output_tokens\":128}}}
{\"type\":\"user\",\"timestamp\":\"2026-04-29T12:00:10.000Z\",\"message\":{\"role\":\"user\",\"content\":[{\"type\":\"tool_use\",\"name\":\"Skill\",\"input\":{\"skill\":\"writing-plans\"}}]}}
";

/// Возвращает `StatusPayload` со всеми типизированными полями + `NamedTempFile`
/// с JSONL-фикстурой. `payload.transcript_path = Some(<tempfile_path>)`.
/// Caller (App::new) хранит NamedTempFile до своего drop'а.
///
/// На ошибку tempfile (read-only `/tmp` etc.) возвращает `None` транскрипта;
/// payload всё равно собирается — preview покажет виджеты без transcript-данных.
#[must_use]
pub fn payload() -> (StatusPayload, Option<NamedTempFile>) {
    let tempfile = write_transcript_fixture();
    let transcript_path = tempfile.as_ref().map(|f| f.path().to_string_lossy().into_owned());

    let p = StatusPayload {
        session_id: "s_demo_abc123".into(),
        model: ModelInfo {
            id: "claude-opus-4-7".into(),
            display_name: "Sonnet 4.7".into(),
        },
        workspace: Workspace {
            current_dir: "/Users/sample/project".into(),
            project_dir: Some("/Users/sample/project".into()),
            added_dirs: None,
        },
        transcript_path,
        cwd: Some("/Users/sample/project".into()),
        version: Some("2.1.119".into()),
        fast_mode: Some(false),
        exceeds_200k_tokens: Some(false),
        output_style: Some(OutputStyle {
            name: Some("default".into()),
        }),
        cost: Some(CostInfo {
            total_cost_usd: Some(1.234),
            total_duration_ms: Some(1_800_000),
            total_api_duration_ms: Some(1_500_000),
            total_lines_added: Some(120),
            total_lines_removed: Some(45),
        }),
        context_window: Some(ContextWindowInfo {
            context_window_size: Some(200_000),
            total_input_tokens: Some(12_500),
            total_output_tokens: Some(3_200),
            current_usage: Some(CurrentUsage::Detailed {
                input_tokens: Some(12_500),
                output_tokens: Some(3_200),
                cache_creation_input_tokens: Some(1_024),
                cache_read_input_tokens: Some(8_192),
            }),
            used_percentage: Some(7.85),
            remaining_percentage: Some(92.15),
        }),
        worktree: Some(Worktree {
            name: Some("wt-demo".into()),
            path: Some("/Users/sample/project/.git/worktrees/wt-demo".into()),
            branch: Some("feature/preview".into()),
            original_cwd: Some("/Users/sample/project".into()),
            original_branch: Some("main".into()),
        }),
        vim: Some(VimState {
            mode: Some("NORMAL".into()),
        }),
        rate_limits: Some(RateLimits {
            five_hour: Some(RateBucket {
                used_percentage: Some(45.0),
                resets_at: Some(now_plus_seconds(4 * 3600)),
            }),
            seven_day: Some(RateBucket {
                used_percentage: Some(8.0),
                resets_at: Some(now_plus_seconds(5 * 86_400)),
            }),
        }),
        effort: None,
        thinking: None,
    };
    (p, tempfile)
}

fn write_transcript_fixture() -> Option<NamedTempFile> {
    let mut f = NamedTempFile::new().ok()?;
    f.write_all(TRANSCRIPT_JSONL.as_bytes()).ok()?;
    f.flush().ok()?;
    Some(f)
}

fn now_plus_seconds(delta: i64) -> i64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(1_745_900_000);
    now + delta
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn payload_has_all_typed_fields() {
        let (p, _f) = payload();
        assert_eq!(p.model.display_name, "Sonnet 4.7");
        assert!(p.cost.is_some());
        assert!(p.context_window.is_some());
        assert!(p.worktree.is_some());
        assert!(p.vim.is_some());
        assert!(p.rate_limits.is_some());
        assert!(p.transcript_path.is_some());
    }

    #[test]
    fn transcript_fixture_writes_5_jsonl_lines() {
        let (_p, f) = payload();
        let f = f.expect("tempfile must be created in this env");
        let body = std::fs::read_to_string(f.path()).unwrap();
        let count = body.lines().filter(|l| !l.is_empty()).count();
        assert_eq!(count, 5, "expected 5 JSONL lines, got {count}");
    }

    #[test]
    fn transcript_lines_are_valid_json() {
        let (_p, f) = payload();
        let f = f.expect("tempfile must be created in this env");
        let body = std::fs::read_to_string(f.path()).unwrap();
        for line in body.lines().filter(|l| !l.is_empty()) {
            let _: serde_json::Value =
                serde_json::from_str(line).expect("each transcript line must be valid JSON");
        }
    }

    #[test]
    fn rate_limits_resets_at_in_future() {
        let (p, _f) = payload();
        let rl = p.rate_limits.unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        assert!(rl.five_hour.unwrap().resets_at.unwrap() > now);
        assert!(rl.seven_day.unwrap().resets_at.unwrap() > now);
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test --features tui --locked tui::sample
```

Expected: 4 PASS.

- [ ] **Step 3: Verify --no-default-features build**

```bash
cargo build --locked --no-default-features
```

Expected: PASS.

- [ ] **Step 4: Lints**

```bash
cargo clippy --locked --features tui --all-targets -- -D warnings
```

Expected: PASS. Если есть warning на `as i64` cast (precision-loss) — позволь через `#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]` в `now_plus_seconds`.

- [ ] **Step 5: Commit**

```bash
git add src/tui/sample.rs
git commit -m "$(cat <<'EOF'
feat(phase-8): T4 sample — inline payload + 5-line JSONL transcript fixture

- tui::sample::payload() -> (StatusPayload, Option<NamedTempFile>)
- All typed fields populated (cost/context/worktree/rate_limits/vim/output_style)
- 5-line JSONL covers tokens, timing, thinking, skills (using-superpowers, writing-plans)
- Tempfile RAII owned by caller (App holds Option<NamedTempFile>)
- 4 unit-tests: payload completeness, JSONL count, JSON validity, future resets_at

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```
