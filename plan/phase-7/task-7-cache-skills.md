# Task 7 — TranscriptStats.skill_names + FORMAT_VERSION 2 + parser merge

**Цель:** Расширить `TranscriptStats` полем `skill_names: Vec<String>` (sorted/unique). Bump `FORMAT_VERSION 1 → 2` (silent reset кэша). Реализовать skill detection в `parser::apply_assistant` через `tool_use.name = "skill_*"` эвристику. Расширить `merge_stats` для union+sort+dedup.

**Files:**
- Modify: `src/cache/jsonl_types.rs:20` — `FORMAT_VERSION 1 → 2`
- Modify: `src/cache/jsonl_types.rs::TranscriptStats` — добавить `skill_names: Vec<String>`
- Modify: `src/cache/parser.rs::ParseState` — `skill_set: BTreeSet<String>`
- Modify: `src/cache/parser.rs::apply_assistant` — skill detection
- Modify: `src/cache/parser.rs::merge_stats` — union+sort+dedup для skill_names
- Modify: `src/cache/fixture.rs::TranscriptBuilder` — добавить `add_assistant_with_tool_uses`

---

- [ ] **Step 1: Write failing test для skill_names в TranscriptStats**

В `src/cache/jsonl_types.rs::tests`:

```rust
#[test]
fn transcript_stats_default_has_empty_skill_names() {
    let s = TranscriptStats::default();
    assert!(s.skill_names.is_empty());
}

#[test]
fn format_version_constant_is_two() {
    assert_eq!(FORMAT_VERSION, 2);
}

#[test]
fn bincode_roundtrip_includes_skill_names() {
    let original = TranscriptStats {
        skill_names: vec!["brainstorming".into(), "executing-plans".into()],
        ..TranscriptStats::default()
    };
    let bytes = bincode::serialize(&original).unwrap();
    let back: TranscriptStats = bincode::deserialize(&bytes).unwrap();
    assert_eq!(back.skill_names, original.skill_names);
}
```

(Старый тест `format_version_constant_is_one` — удалить или заменить на `_is_two`.)

- [ ] **Step 2: Update FORMAT_VERSION + skill_names поле**

В `src/cache/jsonl_types.rs:20`:

```rust
pub const FORMAT_VERSION: u32 = 2;
```

В `pub struct TranscriptStats` после `last_thinking_effort: Option<String>`:

```rust
    pub skill_names: Vec<String>,  // Phase 7 — sorted, unique skill names
```

- [ ] **Step 3: Run jsonl_types tests**

Run: `cargo test --lib cache::jsonl_types`
Expected: PASS — все тесты + 3 новых.

- [ ] **Step 4: Расширить `cache/fixture.rs::TranscriptBuilder`**

Изучить текущий API `TranscriptBuilder` (`src/cache/fixture.rs`). Если у него уже есть метод `add_assistant`, добавить параллельный с поддержкой tool_use:

```rust
impl TranscriptBuilder {
    /// Добавляет assistant-entry с tool_use блоками. Каждый блок: (tool_name, input_json_str).
    pub fn add_assistant_with_tool_uses(
        mut self,
        ts: &str,
        tool_uses: &[(&str, &str)],
    ) -> Self {
        let mut content = Vec::new();
        for (name, input_json) in tool_uses {
            content.push(serde_json::json!({
                "type": "tool_use",
                "name": name,
                "input": serde_json::from_str::<serde_json::Value>(input_json)
                    .unwrap_or(serde_json::Value::Null),
            }));
        }
        let entry = serde_json::json!({
            "type": "assistant",
            "timestamp": ts,
            "message": { "content": content }
        });
        self.append_line(&entry.to_string());  // или соответствующий internal API
        self
    }
}
```

(Адаптировать под фактический shape `TranscriptBuilder` — может быть `self.lines.push(...)` или `self.write_line(...)`.)

- [ ] **Step 5: Write failing test для парсера skill detection**

В `src/cache/parser.rs::tests`:

```rust
#[test]
fn parses_assistant_skill_invocations_into_skill_names() {
    let b = TranscriptBuilder::new()
        .add_assistant_with_tool_uses(
            "2026-04-29T10:00:00Z",
            &[
                ("skill_brainstorming", "{}"),
                ("read_file", "{}"),
                ("skill_executing-plans", "{}"),
            ],
        )
        .add_assistant_with_tool_uses(
            "2026-04-29T10:01:00Z",
            &[("skill_brainstorming", "{}")], // duplicate — must dedup
        );
    let stats = parse_transcript(b.path()).unwrap();
    assert_eq!(stats.skill_names.len(), 2, "dedup'd: {:?}", stats.skill_names);
    assert!(stats.skill_names.contains(&"brainstorming".to_string()));
    assert!(stats.skill_names.contains(&"executing-plans".to_string()));
    // sorted
    let mut sorted = stats.skill_names.clone();
    sorted.sort();
    assert_eq!(stats.skill_names, sorted);
}

#[test]
fn merge_stats_unions_skill_names_sorted_dedup() {
    let mut a = TranscriptStats::default();
    a.skill_names = vec!["alpha".into(), "gamma".into()];
    let mut b = TranscriptStats::default();
    b.skill_names = vec!["beta".into(), "alpha".into()];
    let merged = merge_stats(a, b);
    assert_eq!(
        merged.skill_names,
        vec!["alpha".to_string(), "beta".into(), "gamma".into()]
    );
}
```

- [ ] **Step 6: Run failing parser tests**

Run: `cargo test --lib cache::parser::tests::parses_assistant_skill_invocations_into_skill_names cache::parser::tests::merge_stats_unions_skill_names_sorted_dedup`
Expected: FAIL.

- [ ] **Step 7: Расширить ParseState + apply_assistant в parser.rs**

В `src/cache/parser.rs::ParseState`:

```rust
#[derive(Default)]
struct ParseState {
    last_user_ts: Option<u64>,
    current_block: Option<BillingBlock>,
    stats: TranscriptStats,
    skill_set: std::collections::BTreeSet<String>,  // dedupes incrementally
}
```

В `into_stats`:

```rust
fn into_stats(mut self) -> TranscriptStats {
    if let Some(b) = self.current_block.take() {
        self.stats.blocks.push(b);
    }
    self.stats.skill_names = self.skill_set.into_iter().collect();  // BTreeSet → already sorted
    self.stats
}
```

В `apply_assistant` добавить блок до `if let Some(MessagePayload {..})`:

```rust
fn apply_assistant(state: &mut ParseState, entry: TranscriptEntry, ts: Option<u64>) {
    state.stats.messages = state.stats.messages.saturating_add(1);

    let Some(ts) = ts else { return };
    update_session_bounds(&mut state.stats, ts);

    // Phase 7 — skill detection через content.tool_use.name = "skill_*".
    if let Some(content) = entry
        .message
        .as_ref()
        .and_then(|m| m.content.as_ref())
        .and_then(serde_json::Value::as_array)
    {
        for block in content {
            if block.get("type").and_then(serde_json::Value::as_str) != Some("tool_use") {
                continue;
            }
            if let Some(name) = block.get("name").and_then(serde_json::Value::as_str) {
                if let Some(skill) = name.strip_prefix("skill_") {
                    state.skill_set.insert(skill.to_string());
                }
            }
        }
    }

    if let Some(MessagePayload { usage: Some(u), .. }) = entry.message {
        accumulate_usage(state, u, ts);
    }
    advance_block(state, ts);

    if let Some(ThinkingMeta { effort: Some(level) }) = entry.thinking {
        if !level.is_empty() {
            state.stats.last_thinking_effort = Some(level);
        }
    }
}
```

- [ ] **Step 8: Расширить merge_stats**

В `src/cache/parser.rs::merge_stats` — добавить блок union+sort+dedup для `skill_names`:

```rust
    let mut skill_names = prev.skill_names;
    skill_names.extend(tail.skill_names);
    skill_names.sort_unstable();
    skill_names.dedup();
```

И в финальный struct:

```rust
    TranscriptStats {
        // ...
        last_thinking_effort,
        skill_names,
    }
```

- [ ] **Step 9: Run parser tests**

Run: `cargo test --lib cache::parser`
Expected: PASS — все тесты + 2 новых.

- [ ] **Step 10: Validate format-bump silent reset**

Run: `cargo test --lib cache::store`
Expected: PASS — `format_version_mismatch_triggers_silent_reset` тест Phase 6 продолжает работать (теперь mismatch = old_version != 2).

- [ ] **Step 11: Run full test suite**

Run: `cargo test --locked`
Expected: PASS. Ноль регрессий cache::store / cache::parser / Phase 6 widgets.

- [ ] **Step 12: Commit**

```bash
git add src/cache/jsonl_types.rs src/cache/parser.rs src/cache/fixture.rs
git commit -m "feat(phase-7): T7 — TranscriptStats.skill_names + FORMAT_VERSION 2 + parser skill detection"
```
