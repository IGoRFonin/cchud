# Task 1 — Setup (Cargo bump, scaffolding, DECISIONS)

**Цель:** Подготовить почву для Phase 9. Bump `Cargo.toml` версии до `1.0.0-rc.1`, создать пустые skeleton-файлы для `src/commands/doctor.rs`, директории `npm/`, `scripts/`, `plan/phase-9/`. Добавить запись DECISIONS `D-2026-05-02`. Все существующие тесты остаются зелёными.

**Files:**
- Modify: `Cargo.toml:3` — `version = "0.9.0"` → `version = "1.0.0-rc.1"`
- Create: `src/commands/doctor.rs` (пустой stub с lint deny)
- Modify: `src/commands/mod.rs` — `pub mod doctor;`
- Create: `npm/.gitkeep`
- Create: `npm/templates/.gitkeep`
- Create: `scripts/.gitkeep`
- Create: `plan/phase-9/manual-soak-log.md` (template)
- Modify: `docs/DECISIONS.md` — добавить запись `D-2026-05-02`

---

- [ ] **Step 1: Bump Cargo.toml версии**

Найти строку `version = "0.9.0"` в `Cargo.toml` и заменить на:

```toml
version = "1.0.0-rc.1"
```

- [ ] **Step 2: Создать `src/commands/doctor.rs` skeleton**

```rust
//! `cchud doctor` — environment health check (Phase 9 Task 5).
//!
//! 9 проверок: version, binary path, platform target, color level,
//! hyperlinks, cache dir, claude settings, cchud config, gh CLI.
//!
//! Exit codes:
//!   0 — все checks PASS or SKIP
//!   1 — хоть одна FAIL
//!   2 — нет FAIL, но есть WARN
//!
//! Реализация — Task 5.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::process::ExitCode;

pub fn run(_args: &[String]) -> ExitCode {
    eprintln!("cchud doctor: not yet implemented (Phase 9 Task 5)");
    ExitCode::from(2)
}
```

- [ ] **Step 3: Зарегистрировать модуль в `src/commands/mod.rs`**

Прочитать файл и добавить `pub mod doctor;` рядом с другими `pub mod` объявлениями:

```rust
pub mod configure;
pub mod doctor;       // Phase 9 Task 5
pub mod env_loader;
pub mod import;
pub mod install;
```

(Если `mod.rs` использует `#[cfg(feature = "tui")]` для `configure`/`import` — `doctor` НЕ требует `tui`, держим отдельно без cfg-guard.)

- [ ] **Step 4: Создать пустые директории через `.gitkeep`**

```
npm/.gitkeep
npm/templates/.gitkeep
scripts/.gitkeep
```

- [ ] **Step 5: Создать `plan/phase-9/manual-soak-log.md` шаблон**

```markdown
# Manual soak log — v1.0.0-rc.1

> **Use:** Заполняется во время T13 (RC soak ≥ 24h × 4 environments).
> Все environments должны быть зелёными перед `git tag v1.0.0` (T14).

## Environment 1 — macOS Apple Silicon (e.g. M2 Pro, Sonoma 14.5)
Date: ____
- [ ] `npx --yes cchud@1.0.0-rc.1 install` exits 0
- [ ] `~/.local/bin/cchud` exists, mode 0755
- [ ] `cchud --version` → `1.0.0-rc.1`
- [ ] `cchud doctor` exits 0 (no fail/warn; gh skip без git-pr — OK)
- [ ] `cchud configure` (5+ min real session)
- [ ] Real Claude Code session — statusline renders correctly
Notes: ____

## Environment 2 — macOS Intel (e.g. older Mac или VM)
Date: ____
(checklist same as above)
Notes: ____

## Environment 3 — Linux gnu (Ubuntu 22.04 VM)
Date: ____
- [ ] `npx --yes cchud@1.0.0-rc.1 install` exits 0
- [ ] `curl -fsSL https://raw.githubusercontent.com/IGoRFonin/cchud/main/install.sh | CCHUD_VERSION=1.0.0-rc.1 sh` exits 0
- [ ] `~/.local/bin/cchud` exists, mode 0755
- [ ] `cchud --version` → `1.0.0-rc.1`
- [ ] `cchud doctor` exits 0
Notes: ____

## Environment 4 — Linux musl (Alpine 3.20 docker)
Date: ____
(checklist same as Environment 3)
Notes: ____

## Environment 5 — Windows (Win 11 VM, PowerShell)
Date: ____
- [ ] `npx --yes cchud@1.0.0-rc.1 install` exits 0
- [ ] `%LOCALAPPDATA%\cchud\cchud.exe` exists
- [ ] `cchud --version` → `1.0.0-rc.1`
- [ ] `cchud doctor` exits 0
Notes (Defender SmartScreen?): ____
```

- [ ] **Step 6: Добавить запись `D-2026-05-02` в `docs/DECISIONS.md`**

Прочитать файл и добавить в конец (или согласно существующему ordering) запись:

```markdown
## D-2026-05-02 — Phase 9: Distribution + 1.0.0

**Context:** Шипим `cchud 1.0.0` как production-grade CLI tool с двумя установочными каналами.

**Decisions:**

1. **Channels:** npm (primary, через `npx --yes cchud@1.0.0 install`) + `install.sh` (Mac/Linux secondary). Drop Homebrew (PAT-based formula auto-bump painful), drop crates.io (defer Phase 10).
2. **5 cross-build targets:** `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `x86_64-pc-windows-msvc`. ARM Linux — defer Phase 10.
3. **npm Approach 2 (`optionalDependencies` per platform):** 6 пакетов (`cchud` + 5× `@cchud/cli-<platform>`). Без `postinstall` script — нет network call на install, работает с `--ignore-scripts`.
4. **`npx cchud@<version> install` — primary entry point.** Не `npm i -g cchud`. JS shim spawn'ит native binary с `["install"]`, native binary self-relocates в `~/.local/bin/cchud`.
5. **Self-relocation:** `cchud install` копирует себя в `~/.local/bin/cchud` (Unix) или `%LOCALAPPDATA%\cchud\cchud.exe` (Windows). Идемпотентен. `--no-relocate` flag для разработчиков.
6. **Pin версии в README:** ВСЕГДА конкретная версия (`@1.0.0`), никогда `@latest` — supply-chain mitigation.
7. **Multi-layer security:** npm `--provenance` (sigstore attestations) + npm 2FA (`auth-and-writes`) + `NPM_TOKEN` тип `automation` + GH Actions SHA-pinning.
8. **RC soak gating:** `v1.0.0-rc.1` → npm `dist-tag: next` → ≥ 24h manual soak × 4 envs → `v1.0.0` → npm `dist-tag: latest`. Workflow `npm dist-tag rm cchud latest` rollback при smoke fail.
9. **Smoke-install matrix × 3 (mac/linux/win)** в release.yml — gate между `published` и `actually works`.
10. **`cchud doctor` (9 checks, без Powerline-font detect):** version, binary path, platform target, color level, hyperlinks, cache dir, claude settings, cchud config, gh CLI (conditional). Exit 0/1/2.
11. **Без `install.ps1`** — Windows users имеют npm. PowerShell-ports defer Phase 10.
12. **Reuse Phase 8 `atomic_save` pattern** для settings.json backup `<path>.bak.<unix-ms>`.
```

- [ ] **Step 7: Verify оба `cargo build` зелёные**

```bash
cargo build --locked --all-targets
cargo build --locked --no-default-features --all-targets
```

Expected: оба зелёные. `doctor.rs` stub компилируется (warning о `_args` unused — допустимо).

- [ ] **Step 8: Verify тесты не сломались**

```bash
cargo test --locked
cargo test --locked --no-default-features
```

Expected: ≥220 tests PASS (Phase 8 baseline).

- [ ] **Step 9: Commit**

```bash
git add Cargo.toml src/commands/doctor.rs src/commands/mod.rs npm/ scripts/ plan/phase-9/manual-soak-log.md docs/DECISIONS.md
git commit -m "$(cat <<'EOF'
chore(phase-9): T1 setup — bump 1.0.0-rc.1 + scaffolding

- Cargo: 0.9.0 → 1.0.0-rc.1
- src/commands/doctor.rs stub (T5 implements 9 checks)
- npm/, scripts/, plan/phase-9/ scaffolding
- DECISIONS D-2026-05-02 — channels, packaging, security
EOF
)"
```
