# Task 8 — Snapshots + hyperfine bench (gate)

**Files:**
- Create: `tests/snapshots_git.rs` (≥5 git-сценариев через `GitFixture`)
- Modify: `tests/snapshots.rs` (если уже есть — не трогаем; иначе скип)
- Modify: `benches/configs/phase-5.json` (новый 20-widget bench-конфиг) — путь зависит от Phase 4 структуры
- Create: `benches/phase-5.md` (результаты hyperfine + cache-hit / cache-miss разбивка)
- Modify: `tests/snapshots/...` (snapshot-файлы — будут созданы `cargo insta review`)

## Goal

Финальный гейт перед релизом: **производительность** (p95 < 8 ms на 20-widget config с cache-hit GitPr) и **корректность** (5 git-сценариев в snapshot-тестах).

5 сценариев:
1. **Clean** — свежий репо, один initial commit, без remote, без upstream.
2. **Dirty** — staged + unstaged + untracked файлы.
3. **Conflicts** — merge с конфликтом, 1 conflicted файл.
4. **Fork** — `origin = me/repo`, `upstream = them/repo`.
5. **Detached HEAD** — checkout по SHA, branch = None.

Каждый сценарий рендерится в plain + powerline режимах с конфигом, включающим все 20 git-виджетов.

## Inputs

- T7 закрыт: все 20 git-виджетов работают.
- `cargo test --locked` зелёный.
- `cargo install cargo-insta` (Phase 3 pre-flight).
- `hyperfine ≥ 1.20` (Phase 0).

---

- [ ] **Step 1: Создать `tests/snapshots_git.rs`**

```rust
//! Phase 5 git scenario snapshot tests.
//!
//! Покрывает 5 канонических git-состояний: clean, dirty, conflicts,
//! fork, detached HEAD. Каждое — через GitFixture + 20-widget config.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use cchud::config::Settings;
use cchud::git::fixture::GitFixture;
use cchud::types::payload::{ModelInfo, StatusPayload, Workspace};
use cchud::widgets::{build_widgets, RenderContext};

fn payload(cwd: &str) -> StatusPayload {
    StatusPayload {
        session_id: "snap".into(),
        model: ModelInfo { id: "claude-sonnet-4-6".into(), display_name: "Sonnet 4.6".into() },
        workspace: Workspace { current_dir: cwd.into(), project_dir: None, added_dirs: None },
        transcript_path: None, cwd: None, version: None, fast_mode: None,
        exceeds_200k_tokens: None, output_style: None, cost: None,
        context_window: None, worktree: None, vim: None,
        rate_limits: None, effort: None, thinking: None,
    }
}

/// 20-widget conf — все git-виджеты Phase 5.
fn full_git_settings() -> Settings {
    let json = r#"{
      "lines": [{
        "widgets": [
          { "type": "git-branch" },
          { "type": "git-sha" },
          { "type": "git-root-dir" },
          { "type": "git-status" },
          { "type": "git-changes" },
          { "type": "git-staged" },
          { "type": "git-unstaged" },
          { "type": "git-untracked" },
          { "type": "git-conflicts" },
          { "type": "git-insertions" },
          { "type": "git-deletions" },
          { "type": "git-ahead-behind" },
          { "type": "git-origin-owner" },
          { "type": "git-origin-repo" },
          { "type": "git-origin-owner-repo" },
          { "type": "git-upstream-owner" },
          { "type": "git-upstream-repo" },
          { "type": "git-upstream-owner-repo" },
          { "type": "git-is-fork" }
        ]
      }]
    }"#;
    // Note: исключаем git-pr из snapshot config — он сетевой; покрыт в pr::tests.
    serde_json::from_str(json).unwrap()
}

fn render_all(payload: &StatusPayload, settings: &Settings) -> Vec<String> {
    let ctx = RenderContext::new(payload, settings);
    let widgets = build_widgets(settings);
    widgets
        .iter()
        .map(|w| {
            let id = w.id();
            let val = w.render(&ctx).unwrap_or_default();
            format!("{id} = {val}")
        })
        .collect()
}

#[test]
fn snapshot_clean_repo() {
    let f = GitFixture::new();
    let p = payload(f.path().to_str().unwrap());
    let settings = full_git_settings();
    let out = render_all(&p, &settings).join("\n");
    insta::assert_snapshot!("clean", out);
}

#[test]
fn snapshot_dirty_repo() {
    let f = GitFixture::new();
    f.write_file("staged.txt", "1\n");
    f.git(&["add", "staged.txt"]);
    f.write_file("unstaged.txt", "u\n");
    f.git(&["add", "unstaged.txt"]);
    f.commit("c2");
    f.write_file("unstaged.txt", "u2\n");
    f.write_file("untracked.txt", "x");
    let p = payload(f.path().to_str().unwrap());
    let settings = full_git_settings();
    let out = render_all(&p, &settings).join("\n");
    insta::assert_snapshot!("dirty", out);
}

#[test]
fn snapshot_conflicts_repo() {
    let f = GitFixture::new();
    f.write_file("a.txt", "main\n");
    f.git(&["add", "a.txt"]);
    f.commit("c2");
    f.git(&["checkout", "-b", "feature"]);
    f.write_file("a.txt", "feature\n");
    f.git(&["add", "a.txt"]);
    f.commit("on feature");
    f.git(&["checkout", "main"]);
    f.write_file("a.txt", "main2\n");
    f.git(&["add", "a.txt"]);
    f.commit("on main");
    // Merge — будет конфликт.
    let _ = std::process::Command::new("git")
        .current_dir(f.path())
        .args(["merge", "feature"])
        .output();
    let p = payload(f.path().to_str().unwrap());
    let settings = full_git_settings();
    let out = render_all(&p, &settings).join("\n");
    insta::assert_snapshot!("conflicts", out);
}

#[test]
fn snapshot_fork_repo() {
    let f = GitFixture::new();
    f.add_remote("origin", "git@github.com:me/myrepo.git");
    f.add_remote("upstream", "git@github.com:them/myrepo.git");
    let p = payload(f.path().to_str().unwrap());
    let settings = full_git_settings();
    let out = render_all(&p, &settings).join("\n");
    insta::assert_snapshot!("fork", out);
}

#[test]
fn snapshot_detached_head() {
    let f = GitFixture::new();
    f.write_file("a.txt", "1");
    f.git(&["add", "a.txt"]);
    f.commit("c2");
    f.git(&["checkout", "--detach", "HEAD"]);
    let p = payload(f.path().to_str().unwrap());
    let settings = full_git_settings();
    let out = render_all(&p, &settings).join("\n");
    insta::assert_snapshot!("detached", out);
}
```

Path: `/Users/igor/mp/startup/cchud/tests/snapshots_git.rs`.

> **`fixture` доступ**: `cchud::git::fixture` — `#[cfg(test)]` модуль внутри `cchud` lib. Чтобы он был виден из `tests/`, нужно `pub mod fixture;` без `#[cfg(test)]` ИЛИ exposing через test-only feature. Простейший путь: убрать `#[cfg(test)]` с `pub mod fixture;` в `src/git/mod.rs` и добавить `#[cfg(any(test, feature = "test-fixtures"))]` (или просто оставить публичным — он используется только в тестах). Поправить локально, если linker ругается.
>
> Альтернатива: дублировать `GitFixture` в `tests/common/mod.rs`. Не делаем — DRY важнее.

- [ ] **Step 2: Прогнать тесты, принять snapshot'ы**

```bash
cargo test --locked --test snapshots_git 2>&1 | tail -20
```

Expected первый раз: 5 тестов FAIL — каждый создаёт `.snap.new`. Просмотреть:

```bash
cargo insta review
```

Принять каждый snapshot (`a` — accept). Содержимое должно быть осмысленным:
- `clean`: только git-branch, git-sha, git-root-dir рендерят (остальные пустые)
- `dirty`: git-status `M1 ~1 ?1`, счётчики ненулевые
- `conflicts`: git-conflicts ≥1, git-status содержит `✗N`
- `fork`: git-is-fork = `fork`, оба remote'а заполнены
- `detached`: git-branch пусто, git-sha заполнен

```bash
cargo test --locked --test snapshots_git
```

Expected: 5 PASS.

- [ ] **Step 3: Создать bench-конфиг**

Read `benches/` структуру (Phase 0/3/4 артефакты).

Создать `benches/configs/phase-5-20w.json`:

```json
{
  "lines": [{
    "widgets": [
      { "type": "model" },
      { "type": "git-branch" },
      { "type": "git-sha" },
      { "type": "git-root-dir" },
      { "type": "git-status" },
      { "type": "git-changes" },
      { "type": "git-staged" },
      { "type": "git-unstaged" },
      { "type": "git-untracked" },
      { "type": "git-conflicts" },
      { "type": "git-insertions" },
      { "type": "git-deletions" },
      { "type": "git-ahead-behind" },
      { "type": "git-origin-owner" },
      { "type": "git-origin-repo" },
      { "type": "git-origin-owner-repo" },
      { "type": "git-upstream-owner" },
      { "type": "git-upstream-repo" },
      { "type": "git-upstream-owner-repo" },
      { "type": "git-is-fork" },
      { "type": "git-pr" }
    ]
  }]
}
```

Path: `/Users/igor/mp/startup/cchud/benches/configs/phase-5-20w.json`.

> Точное расположение зависит от структуры Phase 4. Если configs лежат в `benches/configs/` — создаём там. Если иначе — следуем существующей конвенции.

- [ ] **Step 4: Подготовить cache-warmup для GitPr**

Чтобы hyperfine мерил **cache-hit** (а не cache-miss с реальным сетевым вызовом), сначала прогреем кэш:

```bash
cd /Users/igor/mp/startup/cchud
echo '{"session_id":"x","model":{"id":"m","display_name":"M"},"workspace":{"current_dir":"'$(pwd)'"}}' \
  | CCHUD_CONFIG=benches/configs/phase-5-20w.json target/release/cchud > /dev/null
ls -la ~/.cache/cchud/pr-cache.bincode
```

Expected: файл существует. Если `git-pr` не нашёл PR — кэш всё равно создан (`CachedPr { pr: None }`).

Если CI запускает bench без сети — установить заведомо валидный кэш-файл из тестовой фикстуры.

- [ ] **Step 5: Запустить hyperfine**

```bash
cd /Users/igor/mp/startup/cchud
hyperfine --warmup 10 --runs 200 \
  --export-markdown benches/phase-5-raw.md \
  "echo '{\"session_id\":\"x\",\"model\":{\"id\":\"m\",\"display_name\":\"M\"},\"workspace\":{\"current_dir\":\"'$(pwd)'\"}}' | CCHUD_CONFIG=benches/configs/phase-5-20w.json ./target/release/cchud > /dev/null"
```

Expected: median < 8 ms, p95 < 8 ms (`--show-output` для p95 деталей).

Дополнительно — bench без `git-pr` (для baseline сравнения):

```bash
# Создать temp config без git-pr (20 виджетов вместо 21).
jq '.lines[0].widgets |= map(select(.type != "git-pr"))' benches/configs/phase-5-20w.json > /tmp/phase5-no-pr.json
hyperfine --warmup 10 --runs 200 \
  "echo '{\"session_id\":\"x\",\"model\":{\"id\":\"m\",\"display_name\":\"M\"},\"workspace\":{\"current_dir\":\"'$(pwd)'\"}}' | CCHUD_CONFIG=/tmp/phase5-no-pr.json ./target/release/cchud > /dev/null"
```

Сравнить — оверхед `git-pr` (cache-hit) должен быть < 1 ms.

- [ ] **Step 6: Записать `benches/phase-5.md`**

```markdown
# Phase 5 — Hyperfine results

**Date:** 2026-04-XX
**Host:** macOS 24.6 / M2 Pro / 32 GB
**Binary:** `target/release/cchud` (release, LTO, strip)
**Config:** `benches/configs/phase-5-20w.json` (20 git widgets + model)

## Full config (21 widgets, GitPr cache-hit)

| Metric | Value |
|---|---|
| mean | X.XX ms |
| median | X.XX ms |
| stddev | X.XX ms |
| p95 | X.XX ms |
| min | X.XX ms |
| max | X.XX ms |

**Gate:** p95 < 8 ms ✅ / ❌

## Without git-pr (20 widgets — gix only)

| Metric | Value |
|---|---|
| mean | X.XX ms |
| median | X.XX ms |
| p95 | X.XX ms |

**git-pr cache-hit overhead:** delta_p95 = ~Y.YY ms (ожидаем < 1 ms)

## Cluster cost breakdown (диагностика)

Запуск с `cargo run --release --features bench-debug` (если фича добавлена) или ручной анализ:

- `discover()`: ~100 µs
- `parse_head()` + `parse_remotes()`: ~50 µs
- `status_counts()`: ~1.5 ms (gix status)
- `diff_stat()`: ~3 ms (shell-out `git diff --shortstat`) — only if widgets present
- `tracking()`: ~3 ms (shell-out `git rev-list`)
- `lookup_or_fetch()` cache-hit: ~0.4 ms (read + bincode-decode)

**Conclusion:** budget 8 ms покрывается; основной cost — diff_stat + tracking shell-out (~6 ms total). Если убрать diff/tracking widgets — < 4 ms.

## Comparison vs Phase 3 (23 widgets, no git)

| Phase | Widgets | p95 | Note |
|---|---|---|---|
| Phase 3 | 23 (payload only) | ~3-4 ms | baseline |
| Phase 5 | 21 (incl. git+pr) | X.XX ms | + ~5 ms git overhead |

## Comparison vs upstream `ccstatusline` (Node.js)

Запустить ccstatusline на тех же 21-widget config:

```bash
hyperfine --warmup 5 --runs 30 \
  "echo '...' | npx ccstatusline" \
  "echo '...' | ./target/release/cchud"
```

Expected ratio cchud / ccstatusline: ~5-15× быстрее.
```

Path: `/Users/igor/mp/startup/cchud/benches/phase-5.md`.

- [ ] **Step 7: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
cargo insta pending-snapshots 2>&1 | head
```

Expected: все exit 0; pending пусто.

- [ ] **Step 8: Verification**

```bash
ls tests/snapshots/snapshots_git__*.snap | wc -l
ls benches/configs/phase-5-20w.json
ls benches/phase-5.md
grep -c 'p95 < 8 ms ✅' benches/phase-5.md
```

Expected:
```
≥5         (5 snapshot файлов)
exists
exists
1          (gate passed)
```

- [ ] **Step 9: Commit**

```bash
git add tests/snapshots_git.rs tests/snapshots/snapshots_git__*.snap benches/configs/phase-5-20w.json benches/phase-5.md
git commit -m "test(phase-5): T8 snapshots + hyperfine gate

5 git-scenario snapshots via GitFixture: clean / dirty / conflicts /
fork / detached HEAD. Each renders all 19 git widgets (excl. git-pr —
network-bound, covered by mockito tests).

Hyperfine on 21-widget config (incl. git-pr cache-hit): p95 < 8 ms.
benches/phase-5.md captures results + cluster cost breakdown.

Cache-hit overhead from git-pr < 1 ms vs 20-widget baseline.

Task 8/9 of Phase 5.
"
```

## Verification

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

## Definition of Done

- [ ] `tests/snapshots_git.rs` создан с 5 git-сценариями
- [ ] 5 snapshot-файлов committed (`tests/snapshots/snapshots_git__*.snap`)
- [ ] `cargo insta pending-snapshots` пусто
- [ ] `benches/configs/phase-5-20w.json` с 20 git + model widgets (+ git-pr опционально)
- [ ] `benches/phase-5.md` записан с hyperfine результатами
- [ ] p95 < 8 ms на cache-hit GitPr
- [ ] git-pr cache-hit overhead < 1 ms vs baseline
- [ ] Один commit `test(phase-5): T8 snapshots + hyperfine gate ...`

## Files touched

- `tests/snapshots_git.rs` (created)
- `tests/snapshots/snapshots_git__*.snap` (created via insta)
- `benches/configs/phase-5-20w.json` (created)
- `benches/phase-5.md` (created)
- Возможно `src/git/mod.rs` (если убрали `#[cfg(test)]` с `pub mod fixture;`)

## Risks & rollback

- **`fixture` visibility из integration test'а**: `tests/snapshots_git.rs` — отдельный crate, не видит `#[cfg(test)]` модули. Нужно убрать `#[cfg(test)]` с `pub mod fixture;` в `src/git/mod.rs`. Это компромисс: `GitFixture` в release-бинаре (не используется, только публичный API). Альтернатива — feature flag `test-fixtures`.
- **Snapshot flakiness**: SHA в `git-sha` уникален каждый раз (зависит от commit-time). Использовать `[FILTERED]`-redactor через `insta::with_settings!` или фильтровать SHA до фиксированной строки в `render_all`. Если проблемы — добавить редактор:
  ```rust
  insta::with_settings!({filters => vec![(r"sha = [0-9a-f]{7}", "sha = [SHA]")]}, {
      insta::assert_snapshot!(...);
  });
  ```
- **CI вариативность hyperfine**: GitHub Actions runners шумные. Bench запускается локально, в CI только smoke-test (бинарь работает). Документируем в `benches/phase-5.md` хост-spec.
- **GitPr-зависимость в bench**: если кэш-файл не создан (нет сети при CI), bench падает на cache-miss → 200 ms timeout. Митигация: bench-конфиг с pre-сидированным кэш-файлом в `benches/fixtures/pr-cache.bincode`.
- **Conflicts фикстура на Windows**: `git merge` создаёт конфликт-маркеры в файлах с CRLF. Должно работать, но если CI Windows ругается — `core.autocrlf=false` в `GitFixture::new()`.
- **Rollback**: `git revert HEAD` — снимает snapshot и bench.
