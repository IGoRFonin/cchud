# Task 3 — CI workflows + bench + dependabot + actionlint

**Files:**
- Create: `.github/workflows/ci.yml`, `.github/workflows/bench.yml`, `.github/dependabot.yml`, `benches/run.sh`

## Goal

Настроить GitHub Actions: matrix-CI на 3 OS (build/test/clippy/fmt) с предварительной валидацией yaml через actionlint; bench-workflow на PR (hyperfine); Dependabot для cargo + github-actions. Всё проверяется локально перед коммитом.

## Inputs

- Task 2 завершён: `src/main.rs` skeleton + `tests/snapshots.rs` работают локально.
- `benches/samples/` содержит ≥ 3 payload-файлов (для bench-скрипта).
- `gh` CLI установлен (DECISIONS Phase 0: 2.88.1).
- `hyperfine` установлен (DECISIONS Phase 0: 1.20.0).

---

- [ ] **Step 1: Установить `actionlint` локально (если нет)**

```bash
which actionlint || brew install actionlint
actionlint --version
```

Expected: версия (1.6.x+).

Если `brew` недоступен — `go install github.com/rhysd/actionlint/cmd/actionlint@latest` или скачать бинарь с GitHub Releases.

- [ ] **Step 2: Создать директорию `.github/workflows/`**

```bash
mkdir -p .github/workflows
```

- [ ] **Step 3: Создать `.github/workflows/ci.yml`**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  actionlint:
    name: actionlint
    runs-on: ubuntu-latest
    permissions:
      contents: read
    steps:
      - uses: actions/checkout@v4
      - name: Download actionlint
        id: get_actionlint
        run: bash <(curl -fsSL https://raw.githubusercontent.com/rhysd/actionlint/main/scripts/download-actionlint.bash)
        shell: bash
      - name: Run actionlint
        run: ${{ steps.get_actionlint.outputs.executable }} -color
        shell: bash

  test:
    name: test (${{ matrix.os }})
    needs: actionlint
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - name: Build (release)
        run: cargo build --release --locked
      - name: Test
        run: cargo test --locked
      - name: Clippy
        run: cargo clippy --locked -- -D warnings
      - name: Format check
        run: cargo fmt --check
```

Notes:
- `concurrency` отменяет старые run'ы того же PR при новом push.
- `actionlint` job выполняется первым; matrix ждёт его (`needs: actionlint`).
- `dtolnay/rust-toolchain@stable` ставит latest stable (≥ 1.85, edition 2024 ok). `rust-toolchain.toml` НЕ читается этим action — это норм для skeleton (MSRV-job будет добавлен позже когда появятся реальные деп-ограничения).
- `Swatinem/rust-cache@v2` кэширует target/ + ~/.cargo по lockfile hash — экономит 30–60 сек на каждом run.
- На Windows возможны проблемы с длинными путями: задокументировано в Task 4 README troubleshooting; runner имеет `core.longpaths=true` по умолчанию для GitHub-hosted Windows.

- [ ] **Step 4: Создать `.github/workflows/bench.yml`**

```yaml
name: Bench

on:
  pull_request:
    paths:
      - 'src/**'
      - 'Cargo.toml'
      - 'Cargo.lock'
      - 'benches/**'
      - '.github/workflows/bench.yml'

jobs:
  hyperfine:
    name: hyperfine (macos)
    runs-on: macos-latest
    permissions:
      contents: read
      pull-requests: write
      issues: write
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Build (release)
        run: cargo build --release --locked
      - name: Install hyperfine
        run: brew install hyperfine
      - name: Run bench
        run: bash benches/run.sh > bench-results.md
      - name: Comment on PR
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const body = fs.readFileSync('bench-results.md', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: '## hyperfine\n\n' + body
            });
```

Notes:
- `paths` фильтр: bench не запускается на изменения только в docs/ или plan/.
- macOS-only в Phase 1; кросс-OS bench добавляется в Phase 9.
- `actions/github-script@v7` требует `pull-requests: write` permission — задаётся неявно через дефолт `GITHUB_TOKEN` для PR из того же репо. Forks won't post comments — это ок, для них бенч просто не комментит.

- [ ] **Step 5: Создать `benches/run.sh`**

```bash
#!/usr/bin/env bash
set -euo pipefail

if [ ! -x "./target/release/cchud" ]; then
  echo "ERROR: ./target/release/cchud not found. Run 'cargo build --release' first." >&2
  exit 1
fi

# Берём первые 3 семпла (отсортированно для детерминизма)
SAMPLES=$(ls benches/samples/*.json 2>/dev/null | sort | head -3)

if [ -z "$SAMPLES" ]; then
  echo "ERROR: no JSON samples in benches/samples/" >&2
  exit 1
fi

for SAMPLE in $SAMPLES; do
  echo "### $(basename "$SAMPLE")"
  echo
  hyperfine \
    --warmup 20 \
    --runs 200 \
    --shell=none \
    "cat $SAMPLE | ./target/release/cchud" \
    --export-markdown -
  echo
done
```

Notes:
- `--shell=none` — измеряет только бинарь, без shell overhead.
- `set -euo pipefail` — fail на первой ошибке, на unset var, в pipeline.
- `--warmup 20 --runs 200` — соответствует Phase 0 baseline для совместимости.
- Phase 0 baseline.md мерил global ccstatusline за 246.7 мс. cchud skeleton (просто чтение stdin + println) должен быть < 5 мс на M-чипе.

- [ ] **Step 6: Сделать `benches/run.sh` исполняемым**

```bash
chmod +x benches/run.sh
ls -l benches/run.sh
```

Expected: `-rwxr-xr-x ... benches/run.sh`.

- [ ] **Step 7: Создать `.github/dependabot.yml`**

```yaml
version: 2
updates:
  - package-ecosystem: cargo
    directory: /
    schedule:
      interval: weekly
      day: monday
      time: "09:00"
      timezone: Europe/Moscow
    open-pull-requests-limit: 5
    commit-message:
      prefix: "deps"
      include: scope
    labels:
      - dependencies
      - cargo

  - package-ecosystem: github-actions
    directory: /
    schedule:
      interval: weekly
      day: monday
      time: "09:00"
      timezone: Europe/Moscow
    open-pull-requests-limit: 5
    commit-message:
      prefix: "ci"
      include: scope
    labels:
      - dependencies
      - github-actions
```

- [ ] **Step 8: Прогнать `actionlint` локально**

```bash
actionlint .github/workflows/*.yml
```

Expected: exit 0, no output. Если ругается — fix inline (типичные ошибки: missing `permissions:` для `github-script`, неверные shell-expressions). Не подавлять диагностики через `# actionlint-disable`.

- [ ] **Step 9: Прогнать `bench.sh` локально**

```bash
cargo build --release --locked
bash benches/run.sh
```

Expected: 3 markdown-блока вида:
```
### payload-cchud-opus-xlarge.json

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `cat benches/samples/payload-cchud-opus-xlarge.json | ./target/release/cchud` | X.X ± X.X | X.X | X.X | 1.00 |
```

Цифры: cold-start cchud skeleton на M4 Pro должен быть единицы мс (без дисковых IO кроме stdin payload и без deps в hot path).

- [ ] **Step 10: Опциональная валидация `dependabot.yml`**

Если установлен `yamllint`:
```bash
yamllint -d "{rules: {line-length: disable}}" .github/dependabot.yml
```

Expected: exit 0. Если нет yamllint — пропустить, GitHub отвергнет невалидный конфиг с error в Insights → Dependency graph → Dependabot после push.

- [ ] **Step 11: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: всё зелёное (без изменений в Rust-коде с Task 2).

- [ ] **Step 12: Commit**

```bash
git add .github/ benches/run.sh
git commit -m "ci(phase-1): GH Actions matrix + bench + dependabot + actionlint

ci.yml:
- actionlint job validates workflow yaml first
- matrix on ubuntu/macos/windows-latest
- gates: cargo build --release --locked, test, clippy -D warnings, fmt --check
- Swatinem/rust-cache for build cache
- concurrency cancels stale PR runs

bench.yml:
- PR-only, paths-filtered to src/Cargo/benches changes
- hyperfine on macos-latest, posts results as PR comment
- requires actions/github-script@v7

benches/run.sh:
- bash, set -euo pipefail
- iterates first 3 samples (sorted), --warmup 20 --runs 200 --shell=none
- markdown export

dependabot.yml:
- weekly cargo + github-actions updates
- monday 09:00 Europe/Moscow, max 5 PRs each

Task 3/5 of Phase 1.
"
```

## Verification (стандартный гейт + task-specific)

```bash
# Standard gate
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check

# Task-specific
actionlint .github/workflows/*.yml && echo "actionlint ok"
test -x benches/run.sh && echo "bench script executable"
bash benches/run.sh > /tmp/bench-out.md && grep -q "Mean \[ms\]" /tmp/bench-out.md && echo "bench script produces hyperfine output"
test -f .github/dependabot.yml && echo "dependabot present"
```

Expected:
```
actionlint ok
bench script executable
bench script produces hyperfine output
dependabot present
```

## Definition of Done

- [ ] `actionlint .github/workflows/*.yml` exit 0
- [ ] `bash benches/run.sh` отрабатывает локально на 3 семплах
- [ ] `.github/dependabot.yml` присутствует, охватывает cargo + github-actions
- [ ] Один коммит с префиксом `ci(phase-1):`

(Note: реальный CI run на GitHub произойдёт в Task 5 после push. Здесь мы только локально валидируем yaml + скрипт.)

## Files touched

- `.github/workflows/ci.yml` (created)
- `.github/workflows/bench.yml` (created)
- `.github/dependabot.yml` (created)
- `benches/run.sh` (created, +x)

## Risks & rollback

- **`actionlint` ловит warning'и:** обычно справедливые — fix inline. Если warning о deprecated action version (типа `actions/checkout@v3`) — обновить до v4.
- **`hyperfine` падает с timeout на 200 runs:** уменьшить `--runs 100` (точность баланс).
- **`bench.sh` shell-incompatible:** уже `#!/usr/bin/env bash` + `set -euo pipefail`. На Windows runner'е bench не запускается (workflow targets macos-latest only) — ок.
- **`actions/github-script@v7` падает с permission error:** если репо приватное и PR от форка — PR-комментарии могут быть запрещены. Phase 1 push в свой private repo, форков нет — нет проблемы.
- **Dependabot открывает 10+ PR в первый день:** maximum по 5 каждой экосистемы (cargo + actions = до 10). Можно сначала смерджить, потом следующая партия. Альтернатива — снизить limit до 3.
- **Rollback:** `git checkout HEAD~1 -- .github/ benches/run.sh && git clean -fd .github/ benches/run.sh`.
