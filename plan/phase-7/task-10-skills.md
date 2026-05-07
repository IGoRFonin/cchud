# Task 10 — Skills widget

**Цель:** Реализовать `Skills` виджет в `transcript_meta.rs` — читает `TranscriptStats.skill_names.len()` и форматирует `🎯 N`. None если skill_names пуст или transcript отсутствует.

**Files:**
- Modify: `src/widgets/transcript_meta.rs` — добавить `Skills` widget
- Modify: `src/widgets/mod.rs::build_one` — case `WidgetConfig::Skills`

---

- [ ] **Step 1: Write failing tests**

В `src/widgets/transcript_meta.rs::tests` (или новом блоке):

```rust
#[test]
fn skills_returns_count_with_emoji() {
    use crate::cache::TranscriptStats;
    use crate::widgets::test_helpers::payload_no_transcript;

    let p = payload_no_transcript();
    let s = crate::config::default_line();
    let ctx = RenderContext::new(&p, &s);
    let stats = TranscriptStats {
        skill_names: vec![
            "brainstorming".into(),
            "executing-plans".into(),
            "tdd".into(),
        ],
        ..TranscriptStats::default()
    };
    ctx.set_transcript_for_tests(Some(stats));
    assert_eq!(Skills.render(&ctx), Some("🎯 3".into()));
}

#[test]
fn skills_returns_none_when_empty() {
    use crate::cache::TranscriptStats;
    use crate::widgets::test_helpers::payload_no_transcript;

    let p = payload_no_transcript();
    let s = crate::config::default_line();
    let ctx = RenderContext::new(&p, &s);
    ctx.set_transcript_for_tests(Some(TranscriptStats::default()));
    assert!(Skills.render(&ctx).is_none());
}

#[test]
fn skills_returns_none_without_transcript() {
    use crate::widgets::test_helpers::payload_no_transcript;

    let p = payload_no_transcript();
    let s = crate::config::default_line();
    let ctx = RenderContext::new(&p, &s);
    assert!(Skills.render(&ctx).is_none());
}
```

- [ ] **Step 2: Run failing tests**

Run: `cargo test --lib widgets::transcript_meta::tests::skills_returns_count_with_emoji widgets::transcript_meta::tests::skills_returns_none_when_empty widgets::transcript_meta::tests::skills_returns_none_without_transcript`
Expected: FAIL.

- [ ] **Step 3: Реализовать Skills**

В `src/widgets/transcript_meta.rs` добавить:

```rust
pub struct Skills;

impl Widget for Skills {
    fn id(&self) -> &'static str { "skills" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let stats = ctx.transcript()?;
        if stats.skill_names.is_empty() {
            return None;
        }
        Some(format!("🎯 {}", stats.skill_names.len()))
    }
}
```

- [ ] **Step 4: Привязать в build_one**

В `src/widgets/mod.rs::build_one`:

```rust
        // Phase 7 — transcript meta:
        WidgetConfig::Skills => Box::new(transcript_meta::Skills),
```

(Удалить `Skills` из stub-блока T4.)

- [ ] **Step 5: Run tests**

Run: `cargo test --lib widgets::transcript_meta`
Expected: PASS.

- [ ] **Step 6: Run full test suite**

Run: `cargo test --locked`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/widgets/transcript_meta.rs src/widgets/mod.rs
git commit -m "feat(phase-7): T10 — Skills widget reads TranscriptStats.skill_names"
```
