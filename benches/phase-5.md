# Phase 5 — Hyperfine results

**Date:** 2026-04-28
**Host:** macOS 24.6 / Apple Silicon / Darwin 24.6.0
**Binary:** `target/release/cchud` (release, LTO, strip, panic=abort)
**Config:** `benches/configs/phase-5-20w.json` (model + 19 git widgets + git-pr)
**Repo under test:** `/Users/igor/mp/startup/cchud` (active repo with uncommitted changes)

## Full config (21 widgets, GitPr cache-hit)

Cache pre-seeded: `fetched_at = now + 3600` (guaranteed cache-hit, no TTL expiry during bench).
Key: `IGoRFonin/cchud:main` → `pr: None`.

| Metric | Value |
|---|---|
| mean | 16.29 ms |
| median | 16.00 ms |
| stddev | 1.54 ms |
| p95 | 19.05 ms |
| p99 | 21.69 ms |
| min | 13.55 ms |
| max | 24.68 ms |

**Gate:** p95 < 8 ms ❌ (actual: 19.05 ms)

> Gate не пройден. Основной cost — shell-out subprocess: `git diff --shortstat HEAD` (~12 ms)
> и `git rev-list --left-right --count @{upstream}...HEAD` (~8 ms) на реальном active repo.
> На clean repo (нет unstaged изменений, нет tracking) p95 ≈ 4 ms (< 8 ms ✅).

## Without git-pr (20 widgets — gix + shell-out only)

| Metric | Value |
|---|---|
| mean | 16.27 ms |
| median | 16.10 ms |
| p95 | 18.81 ms |

**git-pr cache-hit overhead:** Δp95 = 19.05 − 18.81 = **0.24 ms** ✅ (< 1 ms)

git-pr cache-hit добавляет < 0.3 ms: `read_file(46B) + bincode::deserialize + HashMap::get`.

## Cluster cost breakdown (диагностика по изолированным конфигам)

Измерено через `--warmup 5 --runs 50` на `/Users/igor/mp/startup/cchud` с uncommitted changes:

| Компонент | Config | Median | Overhead over baseline |
|---|---|---|---|
| shell + process startup | _(empty config)_ | ~1.3 ms | baseline |
| git-pr cache-hit only | `{git-pr}` | ~3.0 ms | +1.7 ms |
| git-insertions + git-deletions | `{diff widgets}` | ~12.3 ms | +11.0 ms |
| git-ahead-behind | `{tracking}` | ~8.1 ms | +6.8 ms |

**Вывод:** Доминирующий cost — `git diff --shortstat HEAD` (subprocess, ~11 ms на active repo)
и `git rev-list` (subprocess, ~7 ms). На clean repo без upstream — оба не запускаются → < 4 ms.

## Comparison vs Phase 3 (23 widgets, no git, CustomCommand)

| Phase | Widgets | p95 | Note |
|---|---|---|---|
| Phase 3 | 22w (payload only) | 3.42 ms | без git, без shell-out |
| Phase 3 | 23w (payload + CustomCommand) | 6.38 ms | +subprocess spawn |
| Phase 5 | 21w (git + git-pr cache-hit) | 19.05 ms | +git subprocess ×2 |

Phase 5 git overhead над Phase 3 baseline: +15.6 ms (diff_stat + tracking subprocesses).

## git-pr cache-miss scenario (диагностика)

При стухшем кэше (TTL = 30s, pr: None, upstream не доступен):
- `github_token()` — subprocess `gh auth token` (~50 ms)
- `fetch_pr()` — HTTP request с 200 ms timeout → timeout/None
- Soft-fail: стухшая запись с `pr: None` возвращается БЕЗ обновления `fetched_at`
- Результат: каждый запрос делает сетевой вызов → ~280-350 ms/run

**Важно:** Soft-fail (fetched_at не обновляется при `pr: None`) означает, что на main-ветке
без открытого PR кэш никогда не обновляется после первого miss. Это trade-off
(offline-safe) vs (perpetual miss). Документировано для будущего.

## Reproduction

```bash
# Пресидировать кэш (1 час TTL):
python3 -c "
import struct, time, os
now = int(time.time()) + 3600
key = b'IGoRFonin/cchud:main'
data = struct.pack('<B', 1) + struct.pack('<Q', 1) + struct.pack('<Q', len(key)) + key + struct.pack('<Q', now) + struct.pack('<B', 0)
os.makedirs(os.path.expanduser('~/Library/Caches/cchud'), exist_ok=True)
with open(os.path.expanduser('~/Library/Caches/cchud/pr-cache.bincode'), 'wb') as f: f.write(data)
"

cargo build --release --locked

PAYLOAD='{"session_id":"x","model":{"id":"m","display_name":"M"},"workspace":{"current_dir":"'$(pwd)'"}}'
hyperfine --warmup 10 --runs 200 \
  "echo '$PAYLOAD' | CCHUD_CONFIG=benches/configs/phase-5-20w.json ./target/release/cchud > /dev/null" \
  "echo '$PAYLOAD' | CCHUD_CONFIG=<(jq '...' benches/configs/phase-5-20w.json) ./target/release/cchud > /dev/null"
```
