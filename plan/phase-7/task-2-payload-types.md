# Task 2 — payload::RateLimits типизация

**Цель:** Заменить `payload.rate_limits: Option<serde_json::Value>` (Phase 0) на типизированный `Option<RateLimits>` с двумя buckets (`five_hour`, `seven_day`). Это фундамент для usage-кластера (T8).

**Files:**
- Modify: `src/types/payload.rs:50-57` — заменить `rate_limits: Option<Value>` + добавить `RateLimits` + `RateBucket`

---

- [ ] **Step 1: Добавить failing test для парсинга rate_limits**

В блок `mod tests` в `src/types/payload.rs` добавить:

```rust
#[test]
fn parses_rate_limits_typed() {
    let p: StatusPayload = serde_json::from_str(SAMPLE).unwrap();
    let rl = p.rate_limits.expect("rate_limits present in sample");
    let five = rl.five_hour.expect("five_hour bucket present");
    assert!(five.used_percentage.is_some(), "used_percentage parsed");
    assert!(five.resets_at.is_some(), "resets_at parsed (Unix seconds)");
}

#[test]
fn rate_limits_absent_yields_none() {
    let json = r#"{
        "session_id": "x",
        "model": {"id": "m", "display_name": "M"},
        "workspace": {"current_dir": "/tmp"}
    }"#;
    let p: StatusPayload = serde_json::from_str(json).unwrap();
    assert!(p.rate_limits.is_none());
}

#[test]
fn rate_limits_missing_buckets_yields_none_buckets() {
    let json = r#"{
        "session_id": "x",
        "model": {"id": "m", "display_name": "M"},
        "workspace": {"current_dir": "/tmp"},
        "rate_limits": {}
    }"#;
    let p: StatusPayload = serde_json::from_str(json).unwrap();
    let rl = p.rate_limits.unwrap();
    assert!(rl.five_hour.is_none());
    assert!(rl.seven_day.is_none());
}
```

- [ ] **Step 2: Run failing tests**

Run: `cargo test --lib types::payload::tests::parses_rate_limits_typed types::payload::tests::rate_limits_absent_yields_none types::payload::tests::rate_limits_missing_buckets_yields_none_buckets`
Expected: FAIL (поле `rate_limits` пока `Option<Value>`, нет `RateLimits`/`RateBucket`).

- [ ] **Step 3: Заменить `Option<Value>` на `Option<RateLimits>`**

В `src/types/payload.rs` заменить строку 51-52:

```rust
    // Phase 7 — typed (replaces Option<Value> from Phase 0):
    #[serde(default)]
    pub rate_limits: Option<RateLimits>,
```

И добавить новые типы после `OutputStyle`:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimits {
    #[serde(default)]
    pub five_hour: Option<RateBucket>,
    #[serde(default)]
    pub seven_day: Option<RateBucket>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct RateBucket {
    #[serde(default)]
    pub used_percentage: Option<f64>,
    /// Unix seconds, UTC.
    #[serde(default)]
    pub resets_at: Option<i64>,
}
```

- [ ] **Step 4: Старый тест `parses_real_payload_sample` остаётся валидным**

Строка `assert!(payload.rate_limits.is_some(), "rate_limits must parse");` валидна — тип просто другой. Ничего менять не надо. Проверить, что компилируется.

- [ ] **Step 5: Run all payload tests**

Run: `cargo test --lib types::payload`
Expected: PASS — все 7+ тестов зелёные.

- [ ] **Step 6: Run full test suite (smoke check)**

Run: `cargo test --lib --locked`
Expected: PASS — никаких регрессий, никто не использует `rate_limits` напрямую.

- [ ] **Step 7: Commit**

```bash
git add src/types/payload.rs
git commit -m "feat(phase-7): T2 — typed RateLimits/RateBucket replaces Option<Value>"
```
