# Task 9 — Snapshot suite + hyperfine bench

**Files:**
- Create: `tests/snapshots_transcript.rs` (≥5 фикстур через `insta::glob`)
- Create: `benches/samples/transcripts/empty.jsonl`
- Create: `benches/samples/transcripts/small-fresh.jsonl`
- Create: `benches/samples/transcripts/with-thinking.jsonl`
- Create: `benches/samples/transcripts/block-rollover.jsonl`
- Create: `benches/samples/transcripts/partial-tail.jsonl`
- Create: `benches/samples/payload-with-transcript-small.json` (~50 KB)
- Create: `benches/configs/phase-6-8w.json` (8 transcript widgets)
- Create: `scripts/gen-large-transcript.sh` (~50 MB фикстура для bench)
- Create: `benches/phase-6.md` (hyperfine results)
- Modify: `.gitignore` (`benches/samples/payload-with-transcript-large.json` если ещё не покрыт)
- Create: `tests/snapshots/` директория для `insta` snapshot'ов (insta создаст автоматически)

## Goal

Two-pronged gate перед релизом:

1. **Snapshot suite** через `insta::glob` — 5 фикстур × 8 виджетов = 40 stable assertion'ов на любых regression'ах формата.
2. **Hyperfine performance gate** — cold/warm/append на 50 МБ; cchud-20w (Phase 5 baseline) без регрессии > 10%.

Контракт T9:
1. `tests/snapshots_transcript.rs` загружает payload + transcript fixture для каждого `.jsonl` файла, рендерит 8-widget config, сохраняет result через `insta::assert_snapshot!`.
2. `benches/configs/phase-6-8w.json` — лежит рядом с другими bench-конфигами Phase 5; layout зеркалит существующие.
3. `scripts/gen-large-transcript.sh` — генерирует 50 МБ JSONL за < 2 sec; git-ignored output; используется hyperfine warm-up cycle.
4. `benches/phase-6.md` — таблица с результатами и пометка `MEETS TARGET / BLOCKED`.
5. Не модифицирует production code в `src/`.

## Inputs

- T1–T8 закрыты, все 8 виджетов работают.
- `cargo-insta` установлен.
- `hyperfine ≥ 1.20.0`.

---

- [ ] **Step 1: Создать transcript-фикстуры**

Create `/Users/igor/mp/startup/cchud/benches/samples/transcripts/empty.jsonl`:

```
```

(полностью пустой файл — 0 байт)

Create `/Users/igor/mp/startup/cchud/benches/samples/transcripts/small-fresh.jsonl`:

```jsonl
{"type":"user","timestamp":"2026-01-01T00:00:00Z"}
{"type":"assistant","timestamp":"2026-01-01T00:00:05Z","message":{"usage":{"input_tokens":150,"output_tokens":80,"cache_read_input_tokens":2000,"cache_creation_input_tokens":50}}}
{"type":"user","timestamp":"2026-01-01T00:01:00Z"}
{"type":"assistant","timestamp":"2026-01-01T00:01:08Z","message":{"usage":{"input_tokens":100,"output_tokens":120,"cache_read_input_tokens":2100,"cache_creation_input_tokens":40}}}
{"type":"user","timestamp":"2026-01-01T00:02:00Z"}
{"type":"assistant","timestamp":"2026-01-01T00:02:12Z","message":{"usage":{"input_tokens":200,"output_tokens":150,"cache_read_input_tokens":2300,"cache_creation_input_tokens":60}}}
```

Create `/Users/igor/mp/startup/cchud/benches/samples/transcripts/with-thinking.jsonl`:

```jsonl
{"type":"user","timestamp":"2026-01-01T00:00:00Z"}
{"type":"assistant","timestamp":"2026-01-01T00:00:10Z","message":{"usage":{"input_tokens":100,"output_tokens":50,"cache_read_input_tokens":1000,"cache_creation_input_tokens":0}},"thinking":{"effort":"high"}}
{"type":"user","timestamp":"2026-01-01T00:01:00Z"}
{"type":"assistant","timestamp":"2026-01-01T00:01:15Z","message":{"usage":{"input_tokens":120,"output_tokens":80,"cache_read_input_tokens":1100,"cache_creation_input_tokens":10}},"thinking":{"effort":"max"}}
```

Create `/Users/igor/mp/startup/cchud/benches/samples/transcripts/block-rollover.jsonl`:

```jsonl
{"type":"user","timestamp":"2026-01-01T00:00:00Z"}
{"type":"assistant","timestamp":"2026-01-01T00:00:05Z","message":{"usage":{"input_tokens":100,"output_tokens":50,"cache_read_input_tokens":500,"cache_creation_input_tokens":0}}}
{"type":"user","timestamp":"2026-01-01T05:00:00Z"}
{"type":"assistant","timestamp":"2026-01-01T05:00:05Z","message":{"usage":{"input_tokens":200,"output_tokens":100,"cache_read_input_tokens":600,"cache_creation_input_tokens":10}}}
{"type":"user","timestamp":"2026-01-01T10:00:00Z"}
{"type":"assistant","timestamp":"2026-01-01T10:00:05Z","message":{"usage":{"input_tokens":300,"output_tokens":150,"cache_read_input_tokens":700,"cache_creation_input_tokens":20}}}
```

Create `/Users/igor/mp/startup/cchud/benches/samples/transcripts/partial-tail.jsonl`:

```jsonl
{"type":"user","timestamp":"2026-01-01T00:00:00Z"}
{"type":"assistant","timestamp":"2026-01-01T00:00:05Z","message":{"usage":{"input_tokens":100,"output_tokens":50,"cache_read_input_tokens":500,"cache_creation_input_tokens":0}}}
{"type":"user","timestamp":"2026-01-01T00:01:00Z"}
{"type":"assistant","timestamp":"2026-01-01T00:01
```

> **Note:** последняя строка обрезана (без `}` и без newline) — тест partial-tail.

- [ ] **Step 2: Создать `tests/snapshots_transcript.rs`**

Create `/Users/igor/mp/startup/cchud/tests/snapshots_transcript.rs`:

```rust
//! Transcript snapshot suite — Phase 6 Task 9.
//!
//! `insta::glob` пробегает по `benches/samples/transcripts/*.jsonl` и для
//! каждого файла рендерит 8-widget config. Любые изменения формата вывода
//! зафиксируются как pending snapshot через `cargo insta review`.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use cchud::config::default_line;
use cchud::types::payload::{ModelInfo, StatusPayload, Workspace};
use cchud::types::config::{Line, Settings, WidgetConfig};
use cchud::widgets::{build_widgets, RenderContext};
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
    // Фиксируем now_ms к моменту 2026-01-01T03:00:00Z (внутри первого блока
    // фикстуры block-rollover.jsonl) — чтобы BlockTimer был детерминирован.
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
    insta::glob!("../benches/samples/transcripts/*.jsonl", |path| {
        let rendered = render_for(path);
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        // Очистка кэша между snapshot'ами — детерминизм cold-pat.
        if let Some(home) = dirs::cache_dir() {
            let _ = std::fs::remove_dir_all(home.join("cchud"));
        }
        insta::with_settings!({ snapshot_suffix => name }, {
            insta::assert_snapshot!(rendered);
        });
    });
}
```

Path: `/Users/igor/mp/startup/cchud/tests/snapshots_transcript.rs`.

> **Why `now_ms = 1_767_236_400_000`:** это `2026-01-01T03:00:00Z` — внутри первого 5h-окна `block-rollover` фикстуры (block_start = 0, block_end = 5h = 18_000_000ms). Через 3 часа от начала остаётся 2 часа → `BlockTimer = "⏰ 02:00:00"`. Стабильно для snapshot'а.

> **Why cleanup кэша:** snapshot bear детерминизм; кэш от прошлого запуска может загрязнить результаты, особенно в block-rollover (где порядок merge может зависеть от шага инкремента). Cleanup перед каждым тестом гарантирует cold path.

- [ ] **Step 3: Запустить snapshots first time — pending review**

```bash
cargo test --locked --test snapshots_transcript 2>&1 | tail -10
```

Expected: 5 pending snapshots. (`insta` по умолчанию падает при mismatch / pending в CI. Локально — pending.)

```bash
cargo insta pending-snapshots
```

Должно показать 5 файлов в `tests/snapshots/`.

```bash
cargo insta review
```

Прогнать через interactive review — для каждого файла проверить, что вывод корректный (8 виджетов разделённые ` | `; для `empty.jsonl` все будут `<none>`; для `with-thinking.jsonl` последний — `🧠 max`; etc.). Принять каждый.

- [ ] **Step 4: Создать payload-фикстуру для hyperfine**

Create `/Users/igor/mp/startup/cchud/benches/samples/payload-with-transcript-small.json`:

```json
{
  "session_id": "bench",
  "model": {"id": "claude-sonnet-4-6", "display_name": "Sonnet 4.6"},
  "workspace": {"current_dir": "/tmp"},
  "transcript_path": "benches/samples/transcripts/small-fresh.jsonl"
}
```

> **Note про `transcript_path`:** относительный путь работает потому, что hyperfine запускается из корня репозитория. Если CI запускает иначе — payload должен иметь абсолютный путь, конструируемый скриптом. Альтернатива: добавить `scripts/gen-bench-payload.sh` чтобы экспортировать `$PWD/benches/samples/...`. На macos/linux разработчика — относительный достаточен.

- [ ] **Step 5: Создать `scripts/gen-large-transcript.sh`**

Create `/Users/igor/mp/startup/cchud/scripts/gen-large-transcript.sh`:

```bash
#!/usr/bin/env bash
# Phase 6 Task 9 — generate ~50 MB transcript fixture for hyperfine.
# Output is git-ignored (see .gitignore). Idempotent: skips if file exists
# and has expected size.

set -euo pipefail
cd "$(dirname "$0")/.."

OUT="benches/samples/transcripts/large-50mb.jsonl"
PAYLOAD_OUT="benches/samples/payload-with-transcript-large.json"
TARGET_BYTES=$((50 * 1024 * 1024))

mkdir -p "$(dirname "$OUT")"

# 50 MB ≈ 50_000 sample lines if avg 1 KB each. Generate in batches of 1000.
if [[ -f "$OUT" ]]; then
  current=$(wc -c <"$OUT" | tr -d ' ')
  if [[ "$current" -ge "$TARGET_BYTES" ]]; then
    echo "$OUT already $current bytes — skipping"
    exit 0
  fi
fi

> "$OUT"
i=0
while [[ "$(wc -c <"$OUT" | tr -d ' ')" -lt "$TARGET_BYTES" ]]; do
  ts_user=$(printf '2026-01-01T%02d:%02d:00Z' $(( (i / 60) % 24 )) $(( i % 60 )))
  ts_assistant=$(printf '2026-01-01T%02d:%02d:05Z' $(( (i / 60) % 24 )) $(( i % 60 )))
  in_tokens=$(( 100 + (i % 500) ))
  out_tokens=$(( 50 + (i % 200) ))
  cache_r=$(( 1000 + (i % 5000) ))
  cache_c=$(( i % 100 ))
  printf '{"type":"user","timestamp":"%s"}\n' "$ts_user" >> "$OUT"
  printf '{"type":"assistant","timestamp":"%s","message":{"usage":{"input_tokens":%d,"output_tokens":%d,"cache_read_input_tokens":%d,"cache_creation_input_tokens":%d}},"thinking":{"effort":"high"}}\n' \
    "$ts_assistant" "$in_tokens" "$out_tokens" "$cache_r" "$cache_c" >> "$OUT"
  i=$((i + 1))
done

cat > "$PAYLOAD_OUT" <<EOF
{
  "session_id": "bench-large",
  "model": {"id": "claude-sonnet-4-6", "display_name": "Sonnet 4.6"},
  "workspace": {"current_dir": "/tmp"},
  "transcript_path": "$(pwd)/$OUT"
}
EOF

echo "Generated $OUT ($(wc -c <"$OUT" | tr -d ' ') bytes)"
echo "Payload: $PAYLOAD_OUT"
```

Path: `/Users/igor/mp/startup/cchud/scripts/gen-large-transcript.sh`.

```bash
chmod +x scripts/gen-large-transcript.sh
```

> **Why bash:** проще, чем Rust бинарь. Запускается на macOS/linux одинаково. На Windows CI hyperfine-bench не нужен (release engineering — Linux runner; Windows покрывает только тестами).

- [ ] **Step 6: Создать `benches/configs/phase-6-8w.json`**

Create `/Users/igor/mp/startup/cchud/benches/configs/phase-6-8w.json`:

```json
{
  "version": 1,
  "lines": [
    {
      "widgets": [
        {"type": "tokens-cached"},
        {"type": "tokens-total"},
        {"type": "input-speed"},
        {"type": "output-speed"},
        {"type": "total-speed"},
        {"type": "block-timer"},
        {"type": "session-duration"},
        {"type": "thinking-effort"}
      ]
    }
  ],
  "theme": {}
}
```

Path: `/Users/igor/mp/startup/cchud/benches/configs/phase-6-8w.json`.

- [ ] **Step 7: Запустить hyperfine cold/warm/append**

```bash
# Подготовка: build release, generate large fixture.
cargo build --release --locked
./scripts/gen-large-transcript.sh

# Cold path: wipe cache перед каждым запуском.
hyperfine --warmup 3 --runs 100 \
  --prepare 'rm -rf ~/.cache/cchud/transcript-*.bincode' \
  --export-markdown /tmp/phase-6-cold.md \
  "CCHUD_CONFIG=benches/configs/phase-6-8w.json cat benches/samples/payload-with-transcript-large.json | ./target/release/cchud"

# Warm path: cache present.
hyperfine --warmup 20 --runs 200 \
  --export-markdown /tmp/phase-6-warm.md \
  "CCHUD_CONFIG=benches/configs/phase-6-8w.json cat benches/samples/payload-with-transcript-large.json | ./target/release/cchud"

# Warm + 1 МБ append: append data перед каждым запуском.
hyperfine --warmup 5 --runs 100 \
  --prepare 'head -c 1048576 benches/samples/transcripts/large-50mb.jsonl >> benches/samples/transcripts/large-50mb.jsonl.tmp && mv benches/samples/transcripts/large-50mb.jsonl.tmp benches/samples/transcripts/large-50mb-appended.jsonl 2>/dev/null || true' \
  --export-markdown /tmp/phase-6-append.md \
  "CCHUD_CONFIG=benches/configs/phase-6-8w.json cat benches/samples/payload-with-transcript-large.json | ./target/release/cchud"

# Регрессионный тест: 20-widget Phase 5 config — без regress > 10%.
hyperfine --warmup 10 --runs 200 \
  --export-markdown /tmp/phase-5-baseline.md \
  "CCHUD_CONFIG=benches/configs/phase-5-20w.json cat benches/samples/payload-phase-5.json | ./target/release/cchud"
```

> **Если `CCHUD_CONFIG` env-var не реализован**: cchud скорее всего читает `~/.config/cchud/settings.json`. В этом случае оборачиваем bench в `tempfile`:
> ```bash
> tmp=$(mktemp -d)
> cp benches/configs/phase-6-8w.json "$tmp/settings.json"
> hyperfine ... "XDG_CONFIG_HOME=$tmp cat ... | ./target/release/cchud"
> ```
> Точная команда — см. как Phase 5 T8 запускал hyperfine (`benches/phase-5.md`).

Targets:
- Cold: < 10 ms (mean) ✅
- Warm: < 2 ms (mean) ✅
- Warm + 1 МБ append: < 3 ms (mean) ✅
- cchud-20w (Phase 5): без регресса > 10%

- [ ] **Step 8: Записать результаты в `benches/phase-6.md`**

Create `/Users/igor/mp/startup/cchud/benches/phase-6.md`:

```markdown
# Phase 6 — Hyperfine performance gate

Запуск: 2026-04-XX, mac mini M2 / Linux CI runner.

## Target

Per spec § "Hyperfine":

| Сценарий | Target | Status |
|---|---|---|
| Cold (cache wipe), 50 МБ transcript | < 10 ms mean | <FILL_IN> |
| Warm (cache hit), 50 МБ transcript | < 2 ms mean | <FILL_IN> |
| Warm + 1 МБ append | < 3 ms mean | <FILL_IN> |
| Phase 5 baseline (20w) regress | < 10 % | <FILL_IN> |

## Cold path

```
<paste contents of /tmp/phase-6-cold.md>
```

## Warm path

```
<paste /tmp/phase-6-warm.md>
```

## Warm + 1 МБ append

```
<paste /tmp/phase-6-append.md>
```

## Phase 5 regression check (20w)

```
<paste /tmp/phase-5-baseline.md>
```

Compare mean against committed `benches/phase-5.md`. Allowable delta: < 10%.

## Conclusion

- [ ] All targets met → MERGE OK.
- [ ] Any target missed → BLOCK release; investigate.
```

Path: `/Users/igor/mp/startup/cchud/benches/phase-6.md`.

После запуска шага 7 — paste'нуть фактические числа, обновить `<FILL_IN>` на `MEET` или `MISS`.

- [ ] **Step 9: Update `.gitignore`**

Read `.gitignore` (если существует). Append (if not present):

```gitignore

# Phase 6 — large transcript fixture generated by scripts/gen-large-transcript.sh
benches/samples/transcripts/large-50mb.jsonl
benches/samples/transcripts/large-50mb-appended.jsonl
benches/samples/payload-with-transcript-large.json
```

`file_path`: `/Users/igor/mp/startup/cchud/.gitignore`.

- [ ] **Step 10: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
cargo insta pending-snapshots  # должно быть пусто после Step 3 review
```

Expected: все exit 0; pending snapshots пусто.

- [ ] **Step 11: Verification**

```bash
ls tests/snapshots/snapshots_transcript* | wc -l   # должно быть 5
cat benches/phase-6.md | grep -c MEET   # ≥ 4 (cold, warm, append, regress)
ls benches/samples/transcripts/*.jsonl | wc -l   # должно быть 5+
ls benches/configs/phase-6-8w.json   # должен существовать
```

Expected:
```
≥5
≥4
≥5
benches/configs/phase-6-8w.json
```

- [ ] **Step 12: Commit**

```bash
git add tests/snapshots_transcript.rs tests/snapshots/ \
        benches/samples/transcripts/ benches/samples/payload-with-transcript-small.json \
        benches/configs/phase-6-8w.json benches/phase-6.md \
        scripts/gen-large-transcript.sh .gitignore
chmod +x scripts/gen-large-transcript.sh
git commit -m "test(phase-6): T9 snapshots + hyperfine gate

5 transcript fixtures × 8 widgets × insta::glob → 5 stable snapshots:
- empty.jsonl: 0 messages → all 8 widgets None
- small-fresh.jsonl: 6 messages, ~5 min wall time
- with-thinking.jsonl: thinking.effort=high then max
- block-rollover.jsonl: 3 messages across 2× 5h windows
- partial-tail.jsonl: last line truncated mid-JSON

Hyperfine gates:
- cold (cache wipe), 50MB: <X> ms (<10ms target)
- warm hit, 50MB: <Y> ms (<2ms target)
- warm + 1MB append: <Z> ms (<3ms target)
- Phase 5 20w baseline regression: <W>% (<10% target)

scripts/gen-large-transcript.sh: idempotent ~50MB fixture generator
(git-ignored output). benches/phase-6.md: results table with MEET status.

Task 9/10 of Phase 6. T10 ships 0.4.0."
```

## Verification

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
cargo insta pending-snapshots
```

## Definition of Done

- [ ] `tests/snapshots_transcript.rs` использует `insta::glob` по `benches/samples/transcripts/*.jsonl`
- [ ] 5 фикстур сохранены в `benches/samples/transcripts/` (empty, small-fresh, with-thinking, block-rollover, partial-tail)
- [ ] 5 принятых snapshot'ов в `tests/snapshots/snapshots_transcript@*.snap` (или эквивалентное имя — определяется `insta`)
- [ ] `benches/configs/phase-6-8w.json` создан
- [ ] `scripts/gen-large-transcript.sh` создан, `chmod +x`, idempotent
- [ ] `benches/phase-6.md` содержит результаты hyperfine на 4 сценария: cold / warm / append / Phase-5-regression
- [ ] Cold ≤ 10 ms, warm ≤ 2 ms, warm+append ≤ 3 ms, Phase 5 regress < 10%
- [ ] `.gitignore` исключает large fixture
- [ ] Один commit `test(phase-6): T9 snapshots + hyperfine ...`

## Files touched

- `tests/snapshots_transcript.rs` (created)
- `tests/snapshots/snapshots_transcript@*.snap` (created — committed после `cargo insta review`)
- `benches/samples/transcripts/empty.jsonl` (created)
- `benches/samples/transcripts/small-fresh.jsonl` (created)
- `benches/samples/transcripts/with-thinking.jsonl` (created)
- `benches/samples/transcripts/block-rollover.jsonl` (created)
- `benches/samples/transcripts/partial-tail.jsonl` (created)
- `benches/samples/payload-with-transcript-small.json` (created)
- `benches/configs/phase-6-8w.json` (created)
- `benches/phase-6.md` (created)
- `scripts/gen-large-transcript.sh` (created, +x)
- `.gitignore` (modified)

## Risks & rollback

- **Snapshot нестабилен из-за `now_ms`**: фиксированный `ctx.now_ms = 1_767_236_400_000` — главная защита. Если block-rollover snapshot всё ещё дрейфует — отдельный test override per fixture.
- **`hyperfine` не установлен в CI**: T9 локальный gate, не входит в CI. README: "for releases run scripts/bench.sh locally". CI запускает только `cargo test`.
- **50 MB генерация дольше 2 sec**: bash `printf` slow path. Если медленно — переписать на awk / heredoc batch'ами по 1000 строк (генерация одной строки $(printf) дороже, чем write). Если bash совсем не справляется (>10 sec) — Rust binary `examples/gen_transcript.rs`.
- **`CCHUD_CONFIG` env var не существует**: использовать `XDG_CONFIG_HOME` override (см. Step 7 примечание).
- **Snapshot review требует human attention**: T9 нельзя auto-approve. План указывает запустить `cargo insta review` явно.
- **Phase 5 regression check needs phase-5 config**: предполагается, что `benches/configs/phase-5-20w.json` уже существует (T8 Phase 5). Если нет — создать минимальный 20-widget config для baseline.
- **Rollback**: `git revert HEAD` снимает snapshot suite + bench infra; production code продолжает работать.
