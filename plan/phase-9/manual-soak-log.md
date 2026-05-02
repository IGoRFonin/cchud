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
