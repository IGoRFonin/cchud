# Task 11 — `tui::save::atomic_save`

**Цель:** Атомарный save с backup. Если dest exists → `<dest>.bak.<unix_ts_ms>`. Затем serialize → `<dest>.tmp` → `rename(.tmp, dest)`. Возвращает path к backup'у (`Option`). Никакого UI кода — pure IO.

**Files:**
- Modify: `src/tui/save.rs`

---

- [ ] **Step 1: `atomic_save` + tempdir tests**

```rust
//! Atomic save with backup — Phase 8 Task 11.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::config::Settings;

/// Сохраняет `settings` в `dest` атомарно.
///
/// Алгоритм:
/// 1. Если `dest` существует — копирует в `<dest>.bak.<unix_ts_ms>`.
/// 2. Сериализует pretty-JSON.
/// 3. Пишет в `<dest>.tmp`.
/// 4. `rename(<dest>.tmp, dest)` — atomic on Unix; on Windows MoveFileEx.
///
/// Возвращает `Some(backup_path)` если backup был создан, `None` если файла не было.
/// Ошибки IO/serde пробрасываются.
pub fn atomic_save(settings: &Settings, dest: &Path) -> std::io::Result<Option<PathBuf>> {
    let backup = if dest.exists() {
        let bak = backup_path(dest);
        std::fs::copy(dest, &bak)?;
        Some(bak)
    } else {
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        None
    };

    let json = serde_json::to_string_pretty(settings).map_err(std::io::Error::other)?;
    let tmp = dest.with_extension("json.tmp");
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, dest)?;

    Ok(backup)
}

fn backup_path(dest: &Path) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let mut s = dest.as_os_str().to_owned();
    s.push(format!(".bak.{ts}"));
    PathBuf::from(s)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::types::config::{Line, ModelParams, Settings, ThemeConfig, WidgetConfig, WidgetItem, WidgetStyleOverride};
    use tempfile::tempdir;

    fn sample_settings() -> Settings {
        Settings {
            version: 1,
            lines: vec![Line {
                widgets: vec![WidgetItem {
                    kind: WidgetConfig::Model { params: ModelParams {} },
                    style: WidgetStyleOverride::default(),
                }],
            }],
            theme: ThemeConfig::default(),
        }
    }

    #[test]
    fn writes_to_new_file_no_backup() {
        let dir = tempdir().unwrap();
        let dest = dir.path().join("settings.json");
        let backup = atomic_save(&sample_settings(), &dest).unwrap();
        assert!(backup.is_none());
        assert!(dest.exists());
        // Round-trip parses.
        let body = std::fs::read_to_string(&dest).unwrap();
        let parsed: Settings = serde_json::from_str(&body).unwrap();
        assert_eq!(parsed.version, 1);
    }

    #[test]
    fn creates_backup_when_dest_exists() {
        let dir = tempdir().unwrap();
        let dest = dir.path().join("settings.json");
        std::fs::write(&dest, b"old contents").unwrap();
        let backup = atomic_save(&sample_settings(), &dest).unwrap();
        let bak = backup.unwrap();
        assert!(bak.exists());
        let bak_body = std::fs::read_to_string(&bak).unwrap();
        assert_eq!(bak_body, "old contents");
    }

    #[test]
    fn creates_parent_dirs_when_missing() {
        let dir = tempdir().unwrap();
        let dest = dir.path().join("nested/path/settings.json");
        atomic_save(&sample_settings(), &dest).unwrap();
        assert!(dest.exists());
    }

    #[test]
    fn no_tmp_left_after_save() {
        let dir = tempdir().unwrap();
        let dest = dir.path().join("settings.json");
        atomic_save(&sample_settings(), &dest).unwrap();
        let tmp = dest.with_extension("json.tmp");
        assert!(!tmp.exists(), ".tmp file should be removed by rename");
    }

    #[test]
    fn backup_path_uses_unix_ms_suffix() {
        let p = std::path::PathBuf::from("/tmp/settings.json");
        let bak = backup_path(&p);
        let s = bak.to_string_lossy();
        assert!(s.starts_with("/tmp/settings.json.bak."));
        let suffix = s.strip_prefix("/tmp/settings.json.bak.").unwrap();
        assert!(suffix.parse::<u128>().is_ok(), "suffix must be unix-ms integer");
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test --features tui --locked tui::save
```

Expected: 5 PASS.

- [ ] **Step 3: --no-default-features build + lints**

```bash
cargo build --locked --no-default-features
cargo clippy --features tui --locked --all-targets -- -D warnings
```

Expected: оба PASS.

- [ ] **Step 4: Commit**

```bash
git add src/tui/save.rs
git commit -m "$(cat <<'EOF'
feat(phase-8): T11 save — atomic_save with backup (.bak.<unix-ms>)

- backup existing dest → <dest>.bak.<unix_ts_ms>
- serde_json::to_string_pretty → <dest>.tmp → rename (atomic on Unix/Windows)
- create_dir_all parent if missing
- 5 unit-tests: new-file/backup/nested-parent/tmp-cleanup/suffix-format

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```
