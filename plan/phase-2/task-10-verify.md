# Task 10 — Verification: hyperfine + manual real-CC + sign-off + tag

**Files:**
- Modify: `benches/run.sh` (добавить comparison cchud vs ccstatusline)
- Create: `benches/phase-2.md` (зафиксированный output hyperfine)
- Create: `plan/phase-2/manual-test-log.md` (наблюдения после real-CC теста)
- Modify: `plan/README.md` (Phase 2 → `[x]`, Phase 3 → `[~]`)
- Modify: `docs/DECISIONS.md` (запись Phase 2 — финальное решение по chip-модели, hyperfine-числа)

## Goal

Финализация Phase 2: hyperfine measurement (cchud vs ccstatusline), manual real-Claude-Code тест на ≥5 минут с записью наблюдений, sign-off всех Exit Criteria из README Phase 2, git tag `phase-2-pipeline` (без релиза, маркер фазы).

## Inputs

- Tasks 1–9 закрыты, ВСЕ автоматизированные тесты зелёные локально и на CI.
- `hyperfine` ≥ 1.20.0 в PATH (Phase 0).
- `ccstatusline` доступен в PATH (`which ccstatusline` → `/opt/homebrew/bin/ccstatusline` или аналог). Если нет — `npm install -g @anthropic-ai/ccstatusline` или skip the comparison и используй только cchud.
- Активный Claude Code, можно открыть и работать ≥5 минут.

---

- [ ] **Step 1: Расширить `benches/run.sh` — добавить comparison секцию**

В конец файла (после существующего цикла) добавить:

```bash
# --- Phase 2: cchud vs ccstatusline comparison ---

if command -v ccstatusline >/dev/null 2>&1; then
  COMPARISON_SAMPLE="benches/samples/payload-cchud-sonnet-xlarge.json"
  echo "### Phase 2 comparison: cchud vs ccstatusline"
  echo
  echo "Sample: $(basename "$COMPARISON_SAMPLE")"
  echo
  hyperfine \
    --warmup 20 \
    --runs 200 \
    --shell=none \
    --input "$COMPARISON_SAMPLE" \
    --command-name "cchud" \
    "./target/release/cchud" \
    --command-name "ccstatusline" \
    "ccstatusline" \
    --export-markdown -
  echo
else
  echo "### Phase 2 comparison: SKIPPED (ccstatusline not in PATH)"
fi
```

Edit tool: добавить блок в конец `benches/run.sh`.

- [ ] **Step 2: Запустить bench и сохранить output в `benches/phase-2.md`**

```bash
cargo build --release --locked
bash benches/run.sh > benches/phase-2.md
cat benches/phase-2.md | tail -30
```

Expected: последние строки показывают comparison-таблицу:
```
### Phase 2 comparison: cchud vs ccstatusline

Sample: payload-cchud-sonnet-xlarge.json

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `cchud` | X.X ± X.X | X.X | X.X | 1.00 |
| `ccstatusline` | YYY.Y ± Y.Y | YYY.Y | YYY.Y | NN.NN ± N.NN |
```

Проверить: `cchud` mean < 5 ms (gate). Если > 5 ms — расследовать (не должно быть, бинарь делает почти ничего). Если ccstatusline недоступен — секция написана как SKIPPED, ОК.

Дополнить начало `benches/phase-2.md` ручной информацией о среде:

```bash
# В начало файла вставить (через Edit или вручную):
cat <<EOF > /tmp/phase-2-header.md
# Phase 2 — Hyperfine baseline

**Date:** $(date +%Y-%m-%d)
**Chip:** $(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo "unknown")
**OS:** $(uname -sr)
**Rust:** $(rustc --version)
**cchud version:** $(./target/release/cchud --version)
**ccstatusline version:** $(ccstatusline --version 2>/dev/null || echo "not installed")

EOF
cat /tmp/phase-2-header.md benches/phase-2.md > /tmp/phase-2-final.md
mv /tmp/phase-2-final.md benches/phase-2.md
rm /tmp/phase-2-header.md
```

(Это macOS-only; для Linux замени `sysctl -n machdep.cpu.brand_string` на `lscpu | grep "Model name"` или просто `uname -p`.)

- [ ] **Step 3: Manual real-CC тест**

```bash
# 1. Установить cchud в settings.json (после backup'а текущего)
cp ~/.claude/settings.json ~/.claude/settings.json.pre-phase2-backup
./target/release/cchud install
# Если был чужой statusLine — выйдет с exit 1; в этом случае:
# ./target/release/cchud install --force

# 2. Открыть Claude Code (новое окно или перезагрузка)
# 3. Поработать ≥5 минут: написать пару сообщений, поменять модель если возможно
# 4. Наблюдать statusLine: должна показывать display_name модели (например "Sonnet 4.6")
# 5. Проверить settings.json — все остальные ключи целы:
diff ~/.claude/settings.json.pre-phase2-backup ~/.claude/settings.json
# Expected: только статусLine изменился; mcpServers, theme, всё остальное идентично.

# 6. Откат (опционально):
# cp ~/.claude/settings.json.pre-phase2-backup ~/.claude/settings.json
# rm ~/.claude/settings.json.pre-phase2-backup
```

Если что-то идёт не так в реальном CC — записать в manual-test-log.md, НЕ приставлять чек.

- [ ] **Step 4: Создать `plan/phase-2/manual-test-log.md`**

```markdown
# Phase 2 — Manual real-CC test log

**Date:** YYYY-MM-DD
**Tester:** Igor Fonin
**Chip:** (e.g. Apple M3 Pro)
**OS:** (e.g. macOS 14.6)
**Claude Code version:** (e.g. 2.1.119, см. `cat ~/.claude/version` или Help → About)
**cchud version:** (e.g. 0.0.1)

## Setup
- [ ] Backup `~/.claude/settings.json` → `.pre-phase2-backup`
- [ ] `./target/release/cchud install` (или `--force`)
- [ ] Открыт Claude Code

## Test session
- **Длительность:** XX минут
- **Действий:** N сообщений, M tool-calls
- **Модели:** (если переключал — список)

## Observations
| # | Что | Результат |
|---|---|---|
| 1 | StatusLine отображает display_name модели | ✓ / ✗ |
| 2 | StatusLine обновляется при смене модели | ✓ / ✗ / N/A |
| 3 | Нет видимых лагов в UI Claude Code | ✓ / ✗ |
| 4 | Нет cchud-related записей в Claude Code диагностике / Console.app | ✓ / ✗ |
| 5 | settings.json после теста — diff показывает только statusLine | ✓ / ✗ |

## Notes / issues
(Любые наблюдения, неожиданности, idea'и для будущих фаз.)

## Sign-off
- [ ] Все 5 observations ✓
- [ ] Готов проставить Phase 2 как `[x]` в `plan/README.md`
```

Создать файл с этим шаблоном; заполнить после Step 3.

- [ ] **Step 5: Standard gate финальный**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: всё зелёное. Если что-то красное — НЕ продолжать, открыть отдельную задачу-фикс.

- [ ] **Step 6: Sign-off Exit Criteria из `plan/phase-2/README.md`**

Открыть `plan/phase-2/README.md`, прочитать секцию **Definition of Done всей Фазы 2**. Каждый пункт проверить вручную и проставить `[x]`. Конкретно:

- [ ] Все 10 задач завершены, гейты пройдены — проверить по git log: `git log --oneline | grep "phase-2" | wc -l` должно быть 10
- [ ] Standard gate localhost — Step 5 выше
- [ ] CI зелёный после каждого push'а — `gh run list --branch master --limit 12`
- [ ] `cchud --version` — `./target/release/cchud --version`
- [ ] `cchud install` обрабатывает три кейса — manual в Step 3, install-tests из Task 8
- [ ] AC-007 покрыт — в `tests/snapshots.rs::graceful_fallback_on_broken_json`
- [ ] AC-008 покрыт — в `tests/install.rs::install_preserves_unrelated_keys`
- [ ] Hyperfine p95 < 5ms — `cat benches/phase-2.md | grep "cchud"` показывает Mean<5ms (приближённо p95 ~ Mean+2σ)
- [ ] Manual test log заполнен — `test -s plan/phase-2/manual-test-log.md`
- [ ] git tag — следующий шаг
- [ ] plan/README обновлён — следующий шаг

Edit tool: пройтись по `plan/phase-2/README.md` Definition of Done списку, ставить `[x]` где факт подтверждён.

- [ ] **Step 7: Обновить `plan/README.md` (главный)**

Edit tool на `plan/README.md`:
- `old_string`: `- [~] Фаза 1 — Init (in progress)\n- [ ] Фаза 2 — Pipeline`
- `new_string`: `- [x] Фаза 1 — Init\n- [x] Фаза 2 — Pipeline\n- [~] Фаза 3 — MVP (next)`

(Сверить точные текущие значения через Read; Phase 1 уже мог быть `[x]` в твоём предыдущем коммите.)

- [ ] **Step 8: Обновить `docs/DECISIONS.md`**

Добавить в конец:

```markdown
## Phase 2 → pipeline + first real binary (YYYY-MM-DD)

- 6 design decisions locked in spec `docs/superpowers/specs/2026-04-26-phase-2-pipeline-design.md`
- Hyperfine на (chip): cchud Mean = X.X ms, ccstatusline = YYY.Y ms (~ZZ× speedup)
- AC-001 / AC-007 / AC-008 covered by automated tests
- Manual real-CC test passed: see `plan/phase-2/manual-test-log.md`
- Removed: lexopt (Phase 2 doesn't need named flags)
- Added: serial_test 3, tempfile 3 (dev-only)
- Bin size release: $(ls -la target/release/cchud | awk '{print $5}') bytes (target <5MB)
- Phase 2 tag: phase-2-pipeline
```

(Подставить реальные числа из `benches/phase-2.md` и `ls`.)

- [ ] **Step 9: Финальный commit + tag + push**

```bash
git add benches/run.sh benches/phase-2.md plan/phase-2/manual-test-log.md plan/phase-2/README.md plan/README.md docs/DECISIONS.md
git commit -m "chore(phase-2): verification & sign-off

- benches/run.sh: comparison cchud vs ccstatusline
- benches/phase-2.md: first cchud baseline (chip + measurements)
- manual-test-log.md: real-Claude-Code test passed
- plan/README.md: Phase 2 [x], Phase 3 [~]
- DECISIONS.md: Phase 2 record (hyperfine numbers, removed deps)

All 16 Definition of Done items satisfied. Phase 2 complete.

Task 10/10 of Phase 2.
"

git tag -a phase-2-pipeline -m "Phase 2 — pipeline + first working binary

First end-to-end cchud binary in real Claude Code. Model widget
renders display_name via stdin → parse → render → stdout.
p95 < 5ms on M-series. Architectural skeleton (Widget trait,
RenderContext, Plain renderer, config layer) ready for Phase 3
to add 9 more widgets.

See docs/superpowers/specs/2026-04-26-phase-2-pipeline-design.md
"

git push origin master
git push origin phase-2-pipeline
```

Verify push:
```bash
gh run list --branch master --limit 1
gh release list 2>/dev/null || echo "(no releases — tag-only, no GH release)"
```

Expected: новый CI run на master зелёный (повторный после Task 10 коммита). Tag `phase-2-pipeline` доступен в `gh tag list` или `git ls-remote --tags origin`.

- [ ] **Step 10: Verification — task-specific gate**

```bash
test -s benches/phase-2.md && echo "phase-2.md ok"
test -s plan/phase-2/manual-test-log.md && echo "manual-test-log.md ok"
grep -q '\[x\] Фаза 2' plan/README.md && echo "plan/README updated ok"
git tag -l phase-2-pipeline | grep -q phase-2-pipeline && echo "tag exists ok"
git ls-remote --tags origin phase-2-pipeline 2>/dev/null | grep -q phase-2-pipeline && echo "tag pushed ok"
gh run list --branch master --limit 1 --json conclusion --jq '.[0].conclusion'
```

Expected:
```
phase-2.md ok
manual-test-log.md ok
plan/README updated ok
tag exists ok
tag pushed ok
success
```

## Verification (стандартный гейт)

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

## Definition of Done

- [ ] `benches/phase-2.md` существует, содержит chip+OS+Rust header + hyperfine таблицу
- [ ] cchud Mean < 5ms на локальной M-серии
- [ ] `benches/run.sh` содержит comparison секцию (skipped если ccstatusline отсутствует)
- [ ] Manual real-CC test пройден ≥ 5 минут, лог в `plan/phase-2/manual-test-log.md` со всеми observations ✓
- [ ] `plan/phase-2/README.md` Definition of Done — все пункты `[x]`
- [ ] `plan/README.md` отмечает Phase 2 как `[x]`, Phase 3 как `[~]`
- [ ] `docs/DECISIONS.md` содержит Phase 2 запись с реальными hyperfine-числами
- [ ] git tag `phase-2-pipeline` создан и запушен
- [ ] Последний CI run на master зелёный
- [ ] Финальный коммит `chore(phase-2): verification & sign-off`

## Files touched

- `benches/run.sh` (modified — comparison block)
- `benches/phase-2.md` (created — header + hyperfine output)
- `plan/phase-2/manual-test-log.md` (created — заполненный шаблон)
- `plan/phase-2/README.md` (modified — DoD checkboxes ✓)
- `plan/README.md` (modified — Phase 2 [x])
- `docs/DECISIONS.md` (modified — Phase 2 record)

## Risks & rollback

- **`ccstatusline` не установлен**: comparison-секция скипнется, gate `cchud Mean < 5ms` всё равно проверяется (one-command hyperfine). Acceptable.
- **`hyperfine` mean ≥ 5ms**: реалистично только на очень слабом железе. Расследовать: `hyperfine --runs 1000 --warmup 100 ./target/release/cchud --input ...`. Если систематически — оптимизировать (вероятно `serde_json` с simd-feature). Если разово (cold-start) — записать в риски, накатать в Phase 7.
- **Manual real-CC test провалился (statusLine пустой / лаги / стерр в Console.app)**: НЕ отмечать DoD как пройденный. Открыть отдельный фикс (T11). Чаще всего: payload schema drift (Step 1 fallback в Task 2/9), permission на binary (`chmod +x`), macOS quarantine (`xattr -d`).
- **CI красный после финального push'а**: посмотреть `gh run view <run-id> --log-failed`. Чаще всего — Windows `\r\n` в snapshot'ах (Task 9 fallback `replace("\r\n", "\n")`).
- **`git tag -a` без подписи vs signed**: Phase 1 task-1 git config не настраивал signing — `-a` создаёт annotated unsigned tag, корректно.
- **Rollback final tag** (если нужен): `git tag -d phase-2-pipeline && git push origin :refs/tags/phase-2-pipeline`. Откатывать commit'ы Task 10 — `git revert HEAD` (только если CI красный).
