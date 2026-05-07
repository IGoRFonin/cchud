# Task 8 — Snapshots (≥5 scenarios) + Hyperfine bench gate

**Files:**
- Modify: `tests/snapshots.rs` (≥5 новых сценариев: full-23-widget, worktree+vim, tokens+bar, link+text+symbol+command, default-line регрессия)
- Create: `tests/snapshots/snapshots__*.snap` (новые .snap файлы после `cargo insta review`)
- Create: `benches/configs/phase-3-23w.json` (фиксированный 22-widget config для hyperfine — TerminalWidth исключён ради детерминизма)
- Create: `benches/configs/phase-3-22w-with-cmd.json` (как выше + CustomCommand `echo phase-3`)
- Create: `benches/phase-3.md` (hyperfine markdown export с p95/mean/min/max)
- Modify: `plan/README.md` (если требуется update — обычно T9; в T8 не трогаем)

## Goal

Зафиксировать рендеринг всех 23 виджетов через snapshot-тесты + регрессионный gate p95 < 5 ms на 22/23-widget config через hyperfine.

**Snapshot-сценарии:**

| # | Config | Payload | Что проверяем |
|---|---|---|---|
| 1 | Phase 2 default-line (только `model`) | Phase 0 семплы (4 файла, glob payload-cchud-* / payload-posts-*) | Регрессия Phase 2 (НЕ ломаем) |
| 2 | 22 виджета (всё кроме `TerminalWidth` и `CustomCommand`) | `payload-cchud-sonnet-xlarge.json` | Полный happy path, все Some; CustomCommand отдельно (зависит от env subprocess); TerminalWidth исключён ради детерминизма (no-TTY под cargo test) |
| 3 | Worktree + Vim-only config (5 worktree + VimMode + Worktree alias) | `payload-synthetic-vim-worktree.json` | Worktree кластер + VimMode рендерятся правильно |
| 4 | TokensInput + ContextBar + ContextPercentage + ContextPercentageUsable | `payload-cchud-sonnet-xlarge.json` | Untagged enum + bar formatter + model_context_size lookup |
| 5 | Link + CustomText + CustomSymbol + CustomCommand `echo phase-3` | `payload-cchud-sonnet-xlarge.json` | Static cluster + subprocess (Unix-only — `#[cfg(unix)]` на тесте) |

Сценарий 1 — регрессия Phase 2, использует существующий `insta::glob` (после фильтрации synthetic в T1). Не трогаем.

Сценарии 2–5 — отдельные `#[test]` функции, каждая запускает `cchud` через `assert_cmd::Command::cargo_bin` с specific HOME (через `tempfile`) → `~/.claude/settings.json` → `cchud`-блок. Stdout фиксируется через `insta::assert_snapshot!`.

**Hyperfine gate:**

- 200 runs, 20 warmup, на 22-widget config (без CustomCommand) — детерминированное измерение pure-render perf.
- 100 runs на 23-widget config с CustomCommand `echo phase-3` — ловит вклад subprocess.
- p95 cchud-22w < 5 ms hard gate; cchud-23w-with-cmd → "warning if > 5ms" фиксируем в `benches/phase-3.md`.
- Прирост от Phase 2 baseline (1.3 ms) ≤ +200% (т.е. < ~3.9 ms acceptable для 22-widget).
- Сохранить markdown в `benches/phase-3.md` (committed в репо).

## Inputs

- T1–T7 закрыты, все 23 виджета имеют real impl.
- `cargo-insta` установлен локально.
- `hyperfine` ≥ 1.20.0 в PATH.
- `tempfile` в `[dev-dependencies]` (Phase 2).
- `tests/snapshots/*.snap` — 4 файла Phase 2 + регрессионный glob после фильтрации synthetic.

---

- [ ] **Step 1: Создать config-fixtures для bench**

Create `/Users/igor/mp/startup/cchud/benches/configs/phase-3-22w.json`:

```json
{
  "cchud": {
    "version": 1,
    "lines": [{
      "widgets": [
        {"type": "model"},
        {"type": "version"},
        {"type": "claude-session-id"},
        {"type": "output-style"},
        {"type": "session-name"},
        {"type": "session-clock"},
        {"type": "session-cost"},
        {"type": "context-length"},
        {"type": "context-percentage"},
        {"type": "context-percentage-usable"},
        {"type": "context-bar", "width": 10},
        {"type": "tokens-input"},
        {"type": "tokens-output"},
        {"type": "vim-mode"},
        {"type": "worktree"},
        {"type": "worktree-mode"},
        {"type": "worktree-name"},
        {"type": "worktree-branch"},
        {"type": "worktree-original-branch"},
        {"type": "custom-text", "text": "demo"},
        {"type": "custom-symbol", "symbol": "★"},
        {"type": "link", "url": "https://example.com", "label": "Example"}
      ]
    }],
    "theme": {}
  }
}
```

Это 22 виджета (все кроме TerminalWidth и CustomCommand).

Create `/Users/igor/mp/startup/cchud/benches/configs/phase-3-23w-with-cmd.json`:

Идентичен предыдущему, но добавляет:
```json
        {"type": "custom-command", "command": "echo", "args": ["phase-3"], "timeout_ms": 200}
```

в массив `widgets` после `link`. Итого 23 виджета.

(`mkdir -p benches/configs/` если нужно.)

Verify:
```bash
python3 -m json.tool benches/configs/phase-3-22w.json > /dev/null
python3 -m json.tool benches/configs/phase-3-23w-with-cmd.json > /dev/null
```
Expected: оба exit 0.

- [ ] **Step 2: Расширить `tests/snapshots.rs` — 4 новых сценария**

Open `tests/snapshots.rs`. После существующих 2 функций (`render_default_line_for_phase0_samples`, `graceful_fallback_on_broken_json`) добавить:

```rust
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper: запускает cchud с custom HOME (где `~/.claude/settings.json`
/// — переданный JSON-конфиг) и заданным payload, возвращает stdout.
fn run_with_home_and_payload(home: &PathBuf, payload: &str) -> String {
    let output = Command::cargo_bin("cchud")
        .unwrap()
        .env("HOME", home)
        .env("USERPROFILE", home) // Windows fallback (на Unix игнорируется)
        .write_stdin(payload.to_string())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "non-zero exit: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim_end().to_string()
}

fn write_settings(home: &PathBuf, settings_json: &str) {
    let claude_dir = home.join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    std::fs::write(claude_dir.join("settings.json"), settings_json).unwrap();
}

/// Scenario 2: 22-widget config против реального Phase 0 sonnet-xlarge payload.
/// TerminalWidth исключён (no-TTY под cargo test → None → пустой сегмент).
/// CustomCommand отдельно в Scenario 5.
#[test]
fn scenario_2_full_22_widgets_sonnet_xlarge() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().to_path_buf();
    let settings = include_str!("../benches/configs/phase-3-22w.json");
    write_settings(&home, settings);
    let payload = include_str!("../benches/samples/payload-cchud-sonnet-xlarge.json");
    let stdout = run_with_home_and_payload(&home, payload);
    insta::assert_snapshot!("phase3_full_22w_sonnet", stdout);
}

/// Scenario 3: Worktree-кластер + VimMode на synthetic-семпле.
#[test]
fn scenario_3_worktree_vim() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().to_path_buf();
    let settings = r#"{
        "cchud": {
            "version": 1,
            "lines": [{
                "widgets": [
                    {"type": "model"},
                    {"type": "vim-mode"},
                    {"type": "worktree"},
                    {"type": "worktree-mode"},
                    {"type": "worktree-name"},
                    {"type": "worktree-branch"},
                    {"type": "worktree-original-branch"}
                ]
            }],
            "theme": {}
        }
    }"#;
    write_settings(&home, settings);
    let payload = include_str!("../benches/samples/payload-synthetic-vim-worktree.json");
    let stdout = run_with_home_and_payload(&home, payload);
    insta::assert_snapshot!("phase3_worktree_vim", stdout);
}

/// Scenario 4: Tokens + ContextBar + ContextPercentage + ContextPercentageUsable
/// — лочит untagged enum CurrentUsage и format'ы.
#[test]
fn scenario_4_context_cluster() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().to_path_buf();
    let settings = r#"{
        "cchud": {
            "version": 1,
            "lines": [{
                "widgets": [
                    {"type": "tokens-input"},
                    {"type": "tokens-output"},
                    {"type": "context-length"},
                    {"type": "context-percentage"},
                    {"type": "context-percentage-usable"},
                    {"type": "context-bar", "width": 10}
                ]
            }],
            "theme": {}
        }
    }"#;
    write_settings(&home, settings);
    let payload = include_str!("../benches/samples/payload-cchud-sonnet-xlarge.json");
    let stdout = run_with_home_and_payload(&home, payload);
    insta::assert_snapshot!("phase3_context_cluster", stdout);
}

/// Scenario 5: Static cluster + CustomCommand. Unix-only — Windows
/// behavioural тесты subprocess отложены до Phase 9.
#[cfg(unix)]
#[test]
fn scenario_5_static_and_command() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().to_path_buf();
    let settings = r#"{
        "cchud": {
            "version": 1,
            "lines": [{
                "widgets": [
                    {"type": "custom-text", "text": "demo"},
                    {"type": "custom-symbol", "symbol": "★"},
                    {"type": "link", "url": "https://example.com", "label": "Ex"},
                    {"type": "custom-command", "command": "echo", "args": ["phase-3"], "timeout_ms": 1000}
                ]
            }],
            "theme": {}
        }
    }"#;
    write_settings(&home, settings);
    let payload = include_str!("../benches/samples/payload-cchud-sonnet-xlarge.json");
    let stdout = run_with_home_and_payload(&home, payload);
    insta::assert_snapshot!("phase3_static_and_command", stdout);
}
```

**Note:** `assert_cmd::Command` использует `env()` для child env. `HOME` override → `dirs::home_dir()` подхватит tempdir; `~/.claude/settings.json` будет читаться из tempdir. На Windows `dirs::home_dir()` смотрит `USERPROFILE` — мы передаём оба, чтобы тест работал везде (хотя на Windows custom_command скоринг отключён).

- [ ] **Step 3: Запустить — должны создаться `.snap.new` файлы**

```bash
cargo test --locked --test snapshots
```

Expected:
```
running 6 tests
test graceful_fallback_on_broken_json ... ok
test render_default_line_for_phase0_samples ... ok
test scenario_2_full_22_widgets_sonnet_xlarge ... FAILED
test scenario_3_worktree_vim ... FAILED
test scenario_4_context_cluster ... FAILED
test scenario_5_static_and_command ... FAILED
```

(И в `tests/snapshots/` появились 4 `.snap.new` файла.)

Это нормально. Insta создаёт `.snap.new`, сам тест помечен fail до accept.

- [ ] **Step 4: Ревью через `cargo insta review`**

```bash
cargo insta review
```

Expected: интерактивное TUI, для каждого `.snap.new` показывает diff. Проверить визуально:

- `phase3_full_22w_sonnet` должен содержать `Sonnet 4.6 | 2.1.119 | acf930ea | default | acf930ea-... | 07:58 | $0.72 | 7461 | 25% | 4% | [███░░░░░░░] | inT: 1 | outT: 232 | demo | ★ | <ESC>]8;;https://example.com<ESC>\Example<ESC>]8;;<ESC>\` (приблизительно — точный порядок виджетов из Step 1 config). VimMode и Worktree кластер вернут None, поэтому будут пропущены Plain renderer'ом.
- `phase3_worktree_vim` — `Sonnet 4.6 | NORMAL | wt-feature | WT | wt-feature | feature/synthetic | main`
- `phase3_context_cluster` — что-то типа `inT: 1 | outT: 232 | 7461 | 25% | 4% | [███░░░░░░░]`
- `phase3_static_and_command` — `demo | ★ | <link> | phase-3`

Если что-то не сходится с ожиданием — это либо реальный баг (return to соответствующего T1–T7), либо спецификация неточна. Принимать только если значения логичны.

Accept all через `A` (capital A).

`.snap.new` → `.snap`.

**Альтернатива для CI / non-interactive:**
```bash
INSTA_UPDATE=always cargo test --locked --test snapshots
```
(Только для первичной генерации! Не использовать в дальнейшем — заглушит реальные расхождения.)

- [ ] **Step 5: Verify snapshots committed and stable**

```bash
ls tests/snapshots/ | grep -c '\.snap$'
ls tests/snapshots/ | grep -c '\.snap\.new$'
cargo test --locked --test snapshots
```

Expected:
```
≥8       (4 Phase 2 + 4 Phase 3 — могут быть и больше, если insta создаёт .snap для каждого scenario_*)
0
running 6 tests
... all 6 passed
```

- [ ] **Step 6: Запустить hyperfine — 22-widget bench**

```bash
mkdir -p benches/results
hyperfine \
  --warmup 20 --runs 200 \
  --export-markdown benches/results/phase-3-22w.md \
  --command-name "cchud-22w" \
  "HOME=/tmp/cchud-bench cat benches/samples/payload-cchud-sonnet-xlarge.json | ./target/release/cchud" \
  --command-name "cchud-default" \
  "cat benches/samples/payload-cchud-sonnet-xlarge.json | ./target/release/cchud"
```

**Pre-step:** настроить bench HOME с 22-widget config:

```bash
mkdir -p /tmp/cchud-bench/.claude
cp benches/configs/phase-3-22w.json /tmp/cchud-bench/.claude/settings.json
```

(При запуске `HOME=/tmp/cchud-bench cchud` — load() возьмёт `phase-3-22w.json` и отрендерит 22 виджета.)

Re-run hyperfine после copy.

Expected output:
```
Benchmark 1: cchud-22w
  Time (mean ± σ):       2.0 ms ±  0.2 ms
  ...
  p95: 2.4 ms
  Range (min … max):     1.7 ms …  3.1 ms

Benchmark 2: cchud-default
  Time (mean ± σ):       1.4 ms ±  0.1 ms
  ...
  p95: 1.6 ms
```

(Цифры приблизительны для M-серии. Если p95 cchud-22w > 5ms — gate провален; перейти к Risks секции.)

- [ ] **Step 7: Запустить hyperfine — 23-widget с CustomCommand**

```bash
mkdir -p /tmp/cchud-bench-cmd/.claude
cp benches/configs/phase-3-23w-with-cmd.json /tmp/cchud-bench-cmd/.claude/settings.json

hyperfine \
  --warmup 10 --runs 100 \
  --export-markdown benches/results/phase-3-23w-with-cmd.md \
  --command-name "cchud-23w-cmd" \
  "HOME=/tmp/cchud-bench-cmd cat benches/samples/payload-cchud-sonnet-xlarge.json | ./target/release/cchud"
```

Expected: Time mean ~3–4 ms, p95 ~4–5 ms. CustomCommand subprocess `echo` — типично 0.5–1.5 ms.

Если p95 > 5ms — это "warning", не hard gate. Документируем в `benches/phase-3.md`.

- [ ] **Step 8: Создать `benches/phase-3.md` — comprehensive report**

Create `/Users/igor/mp/startup/cchud/benches/phase-3.md`:

```markdown
# Phase 3 — Hyperfine Benchmarks

> Generated 2026-04-26 on macOS 24.x / arm64 (M-серия).
> Toolchain: rustc 1.85+ (release profile, lto = true, codegen-units = 1).

## Workloads

| Bench | Config | Widgets | Subprocess |
|---|---|---|---|
| cchud-default | `default-line()` (1 виджет) | 1 | no |
| cchud-22w | `benches/configs/phase-3-22w.json` | 22 (без TerminalWidth, CustomCommand) | no |
| cchud-23w-cmd | `benches/configs/phase-3-23w-with-cmd.json` | 23 (включая `echo phase-3`) | yes |

Все запуски: `cat <payload.json> | ./target/release/cchud` с `HOME` override → `~/.claude/settings.json` → конкретный config.

## Results (mean ± σ, p95)

(Заполнить из `benches/results/phase-3-22w.md` и `benches/results/phase-3-23w-with-cmd.md`.)

| Bench | Mean | p95 | Min | Max | Gate | Status |
|---|---|---|---|---|---|---|
| cchud-default | TBD ms | TBD ms | TBD ms | TBD ms | < 5 ms | ✓ |
| cchud-22w | TBD ms | TBD ms | TBD ms | TBD ms | < 5 ms | ✓ / ✗ |
| cchud-23w-cmd | TBD ms | TBD ms | TBD ms | TBD ms | warning if > 5 ms | ✓ / ⚠ |

## Phase 2 baseline reference

Phase 2 hyperfine (`benches/phase-2.md`): mean ≈ 1.3 ms, p95 ≈ 1.6 ms, для 1-widget config.

## Phase 3 budget breakdown (worst-case)

| Стадия | Бюджет |
|---|---|
| Cold-start (binary load + dyld macOS) | ~1–2 ms |
| stdin + JSON parse (envelope ~1.5 KB) | ~0.4 ms |
| config load + parse | ~0.3 ms |
| 22 widget renders (clone/format) | ~0.3 ms |
| 1 subprocess CustomCommand с `echo` | ~0.5–1.5 ms (process spawn dominates) |
| println | ~0.05 ms |
| Worst-case итого | ~3.5–4.5 ms |

## Notes

- **TerminalWidth** исключён из bench-config: под `cat ... | cchud` нет TTY → виджет рендерит None, скоринг искажается. Покрыт unit-тестами.
- **CustomCommand с `echo`**: spawn доминирует. Если бюджет 5 ms превышен, README документирует "warning if CustomCommand используется".
- **Insta snapshot-тесты** для 22-widget рендера лочат корректность; bench только меряет скорость.

## Reproduction

```bash
# 22-widget gate (p95 < 5 ms)
mkdir -p /tmp/cchud-bench/.claude
cp benches/configs/phase-3-22w.json /tmp/cchud-bench/.claude/settings.json
hyperfine --warmup 20 --runs 200 \
  "HOME=/tmp/cchud-bench cat benches/samples/payload-cchud-sonnet-xlarge.json | ./target/release/cchud"

# 23-widget с subprocess
mkdir -p /tmp/cchud-bench-cmd/.claude
cp benches/configs/phase-3-23w-with-cmd.json /tmp/cchud-bench-cmd/.claude/settings.json
hyperfine --warmup 10 --runs 100 \
  "HOME=/tmp/cchud-bench-cmd cat benches/samples/payload-cchud-sonnet-xlarge.json | ./target/release/cchud"
```
```

(Заполнить `TBD ms` ячейки реальными цифрами из `benches/results/*.md` — там mean/min/max; p95 либо посчитать вручную, либо взять `--export-json` и пропустить через `jq`. Hyperfine markdown export не включает p95 по умолчанию — добавить `--show-output` опционально, или использовать json export + jq.)

**Альтернативный gate-расчёт без `--export-json`:** mean + 2σ ≈ p95 для нормального распределения. Если distribution skewed — взять max-25% как proxy.

- [ ] **Step 9: Проверить gate**

```bash
# Прочитать mean из markdown (приблизительно):
grep -E '^\| `cchud-22w` \|' benches/results/phase-3-22w.md
```

Если mean+2σ ≤ 4 ms → p95 < 5 ms — gate ✓.
Если mean+2σ > 5 ms → red flag, проверить что:
- Запускаемся под release profile (`./target/release/cchud`, не `./target/debug/`).
- Есть `lto = true` в `[profile.release]`.
- Бинарь stripped (`strip = true`).
- На M-серии (Apple Silicon), не Rosetta или x86_64-сборка.

- [ ] **Step 10: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: всё зелёное. Тестов суммарно ≥113 (109 после T7 + 4 snapshot scenarios; T8 не пишет unit-тестов).

- [ ] **Step 11: Verification — task-specific gate**

```bash
ls tests/snapshots/ | grep -c '\.snap$'
ls tests/snapshots/ | grep -c '\.snap\.new$'
ls benches/configs/ | wc -l
test -f benches/phase-3.md && echo "phase-3.md ok"
ls benches/results/ | wc -l
grep -c 'scenario_2_full_22_widgets_sonnet_xlarge' tests/snapshots.rs
grep -c 'scenario_5_static_and_command' tests/snapshots.rs
```

Expected:
```
≥8
0
2          (phase-3-22w.json + phase-3-23w-with-cmd.json)
phase-3.md ok
2          (phase-3-22w.md + phase-3-23w-with-cmd.md)
1
1
```

- [ ] **Step 12: Commit**

```bash
git add tests/snapshots.rs tests/snapshots/ \
        benches/configs/ benches/phase-3.md benches/results/
git commit -m "test(phase-3): T8 snapshots (5 scenarios) + hyperfine gate

5 snapshot-сценариев:
1. Phase 2 default-line (insta::glob, регрессия — оригинал)
2. Full 22-widget (без TerminalWidth/CustomCommand) — лочит весь
   контент-слой Phase 3
3. Worktree+Vim cluster на synthetic-семпле
4. Context cluster (TokensInput/Output, Bar, Percentage, Usable)
5. Static + CustomCommand (Unix-only)

HOME override через tempfile → ~/.claude/settings.json контролирует
config; reproducible cross-platform.

Hyperfine:
- cchud-22w gate: p95 < 5 ms (M-серия)
- cchud-23w-cmd: ~3–4 ms mean (subprocess spawn доминирует)
- benches/phase-3.md — markdown report

Phase 2 baseline (1.3 ms mean) → Phase 3 22w (~2 ms mean): прирост ~50%,
в budget +200%.

Task 8/9 of Phase 3.
"
```

## Verification (стандартный гейт)

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

## Definition of Done

- [ ] `benches/configs/phase-3-22w.json` и `phase-3-23w-with-cmd.json` созданы и валидны
- [ ] `tests/snapshots.rs` содержит 4 новых сценария (`scenario_2..5`)
- [ ] `tests/snapshots/` содержит ≥8 `.snap` файлов, 0 `.snap.new`
- [ ] Все 6 тестов в `tests/snapshots.rs` зелёные после `cargo insta review`
- [ ] Phase 2 default-line snapshot'ы НЕ изменились (регрессии нет)
- [ ] `benches/phase-3.md` создан с заполненными числами и Reproduction секцией
- [ ] `benches/results/phase-3-22w.md` и `phase-3-23w-with-cmd.md` committed
- [ ] cchud-22w mean+2σ ≤ 4 ms (proxy для p95 < 5 ms gate)
- [ ] Один commit `test(phase-3): T8 snapshots + hyperfine gate`

## Files touched

- `tests/snapshots.rs` (modified, +4 scenarios + helpers)
- `tests/snapshots/*.snap` (created, ≥4 новых)
- `benches/configs/phase-3-22w.json` (created)
- `benches/configs/phase-3-23w-with-cmd.json` (created)
- `benches/phase-3.md` (created)
- `benches/results/phase-3-22w.md` (created)
- `benches/results/phase-3-23w-with-cmd.md` (created)

## Risks & rollback

- **`HOME=/tmp/...` не работает на Windows CI**: на Windows `dirs::home_dir()` смотрит `USERPROFILE`. Тесты передают оба через `env()`. Snapshot scenario_5 под `#[cfg(unix)]` потому что CustomCommand `echo`. Snapshots 2/3/4 работают на любой платформе.
- **`tempfile::TempDir` race в `serial_test`-mode не нужен**: каждый snapshot-тест получает уникальный tempdir, нет общего HOME. Без `serial_test` ОК.
- **Snapshot-тест 4 (context_cluster) даёт `4%` или `3%` для `ContextPercentageUsable`?** 7461 / 200_000 ≈ 0.0373 = 3.73% → round = 4. Лочим как 4% в snapshot. Если в insta diff увидишь 3% — это ошибка implement'а T4 (round vs floor).
- **OSC 8 byte sequence в snapshot**: `insta` сериализует `\x1b` как литерал. Snapshot будет содержать `\x1b]8;;https://example.com\x1b\\Example\x1b]8;;\x1b\\` (raw). Cross-platform OK, потому что `\x1b` — single byte.
- **Hyperfine падает если `./target/release/cchud` не существует**: pre-step `cargo build --release --locked`.
- **p95 cchud-22w > 5 ms**: изучить что добавилось — vidget render — clone-dominated; format!() на f64 — fast; Stdio::piped в CustomCommand доминирует. Если 22w (без CC) >5ms — есть проблема. Probable causes: debug-build, JSON-парсинг envelope тяжёлый, config-парсинг тяжёлый. Phase 6 vector: switch to `sonic-rs` для JSON. Phase 3 mitigation: документировать в `benches/phase-3.md` что budget не достигнут на M-1 vs M-3, перенести оптимизацию в Phase 6/7.
- **`insta::assert_snapshot!` requires `cargo-insta` для review**: документируем в README. CI флоу: `cargo test --locked --test snapshots` без INSTA_UPDATE — если есть `.snap.new`, fail; снапшоты лочат состояние.
- **`include_str!("../benches/configs/...")` не находит**: путь relative к test-файлу (`tests/snapshots.rs`); `../benches/configs/` корректно.
- **Subprocess `echo` в Snapshot 5 даёт разный output на BSD vs GNU**: BSD `echo` без `-n` всё равно дописывает \n; GNU `/bin/echo` тоже. `trim()` убирает. ОК.
- **Rollback**: `git revert HEAD` — снимает scenarios + .snap файлы + bench artifacts. Phase 2 snapshot'ы и all unit-тесты не задеваются.
