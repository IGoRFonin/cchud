# Task 1 — Setup (sysinfo, skeleton, DECISIONS)

**Цель:** Подготовить почву для Phase 7. Добавить `sysinfo` dep, создать пустые модули-skeleton (5 файлов), обновить `mod.rs`-индексы, записать DECISIONS-запись D-2026-04-29.

**Files:**
- Modify: `Cargo.toml:17-50` (добавить `sysinfo` dep)
- Create: `src/widgets/env.rs`
- Create: `src/widgets/usage.rs`
- Create: `src/commands/env_loader.rs`
- Create: `src/render/flex.rs`
- Create: `src/util/format_memory.rs`
- Create: `src/util/format_duration_long.rs`
- Modify: `src/widgets/mod.rs:12-30` (добавить `pub mod env; pub mod usage;`)
- Modify: `src/commands/mod.rs` (добавить `pub mod env_loader;`)
- Modify: `src/render/mod.rs:152-156` (добавить `pub mod flex;`)
- Modify: `src/util/mod.rs` (добавить `pub mod format_memory; pub mod format_duration_long;`)
- Modify: `docs/DECISIONS.md` (добавить запись)

---

- [ ] **Step 1: Добавить sysinfo dep в Cargo.toml**

В блок `[dependencies]` после строки `time = { version = "0.3", ... }`:

```toml
# Phase 7 — FreeMemory widget (no system-side process scan, just memory).
sysinfo = { version = "0.32", default-features = false, features = ["system"] }
```

- [ ] **Step 2: Создать пустые модули-skeleton**

Все стартовые файлы:

```rust
// src/widgets/env.rs
//! Environment widgets — Phase 7 Task 9.
//! ClaudeAccountEmail, FreeMemory.

#![deny(clippy::unwrap_used, clippy::expect_used)]
```

```rust
// src/widgets/usage.rs
//! Usage cluster widgets — Phase 7 Task 8.
//! SessionUsage, WeeklyUsage, BlockResetTimer, WeeklyResetTimer.
//! Источник: payload.rate_limits (Decision 1, payload-only).

#![deny(clippy::unwrap_used, clippy::expect_used)]
```

```rust
// src/commands/env_loader.rs
//! ~/.claude.json reader — process-wide OnceLock cache.

#![deny(clippy::unwrap_used, clippy::expect_used)]
```

```rust
// src/render/flex.rs
//! Flex-mode width truncation — Phase 7 Task 11.

#![deny(clippy::unwrap_used, clippy::expect_used)]
```

```rust
// src/util/format_memory.rs
//! Auto-unit byte formatter (Kb/Mb/Gb/Tb).

#![deny(clippy::unwrap_used, clippy::expect_used)]
```

```rust
// src/util/format_duration_long.rs
//! Duration formatters — short ("4h32m") and long ("5d 14h") variants.

#![deny(clippy::unwrap_used, clippy::expect_used)]
```

- [ ] **Step 3: Подключить модули в parent mod.rs файлах**

В `src/widgets/mod.rs` после `pub mod context;` (≈ строка 12):

```rust
pub mod env;       // Phase 7 — env cluster
pub mod usage;     // Phase 7 — usage cluster
```

В `src/commands/mod.rs`:

```rust
pub mod env_loader;  // Phase 7
```

В `src/render/mod.rs` рядом с `pub mod themes;`:

```rust
pub mod flex;  // Phase 7
```

В `src/util/mod.rs`:

```rust
pub mod format_memory;          // Phase 7
pub mod format_duration_long;   // Phase 7
```

- [ ] **Step 4: Добавить запись в DECISIONS**

В `docs/DECISIONS.md` добавить раздел:

```markdown
## D-2026-04-29 — Phase 7 ключевые решения

**Контекст:** Реализация Phase 7 (релиз 0.5.0): 7 виджетов + per-widget overrides + 9 global theme settings + multi-line.

**Решения:**
1. **Источник для usage-кластера — payload-only** (`rate_limits.{five_hour, seven_day}`). Без HTTP/auth/keychain. CC ≥ 2.1.x шлёт `rate_limits` напрямую (подтверждено в `benches/samples/payload-cchud-sonnet-xlarge.json`).
2. **Per-widget overrides — `WidgetItem` wrapper.** `Vec<WidgetItem>` вместо `Vec<WidgetConfig>` в `Line`. Один flatten на конфиг.
3. **Multi-line — caller-loop в `main.rs`.** `Renderer::render_line(segs, &mut state)` остаётся single-line.
4. **`auto_align` — sentinel `WidgetConfig::AlignRight`.** 61-й вариант enum, не считается в "60 widgets".
5. **`compact_threshold < term_width` форсит `minimalist_mode`.**
6. **Skills tracking — `TranscriptStats.skill_names: Vec<String>`** (sorted/unique). `FORMAT_VERSION 1 → 2` (silent reset).
7. **`sysinfo = "0.32"`** для `FreeMemory`. `default-features = false`, `features = ["system"]`. `refresh_memory()` точечно (~50 µs macOS).
8. **`OnceLock<Option<ClaudeJson>>`** для `claude_account_email()`. Файл ≤ 10 KB; читается один раз за процесс.

**Связанные документы:** `docs/superpowers/specs/2026-04-29-phase-7-other-widgets-design.md`, `plan/phase-7-other-widgets.md` (outline).
```

- [ ] **Step 5: Проверить сборку**

Run: `cargo build --release --locked`
Expected: PASS (новые модули пусты, lints не падают на attribute).

Run: `ls -la target/release/cchud && du -h target/release/cchud`
Expected: размер < 8.5 MB.

- [ ] **Step 6: Run existing tests (smoke)**

Run: `cargo test --locked`
Expected: PASS — никаких регрессий, только pre-existing tests.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock src/widgets/{env,usage}.rs src/commands/env_loader.rs \
        src/render/flex.rs src/util/format_{memory,duration_long}.rs \
        src/widgets/mod.rs src/commands/mod.rs src/render/mod.rs src/util/mod.rs \
        docs/DECISIONS.md
git commit -m "chore(phase-7): T1 setup — sysinfo dep, skeleton modules, DECISIONS"
```
