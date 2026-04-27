//! Git domain — lazy `GitInfo` через gix; виджеты Phase 5 — тонкие
//! getter'ы над этой структурой.
//!
//! Контракт:
//! - `discover(cwd)` пытается найти репо начиная с `cwd` вверх. None если
//!   за границу маунта или до `/`.
//! - `head` и `remotes` парсятся eagerly при `discover()` (стоимость ~50 µs).
//! - `status_counts` и `diff_stat` — `OnceCell`, считаются только при
//!   запросе соответствующим виджетом (T3, T4).
//! - `RenderContext::git()` обеспечивает однократный `discover()` за render.

#![deny(clippy::unwrap_used, clippy::expect_used)]
// T1 scaffolding — all structs/functions used in T2–T5.
#![allow(dead_code)]

use std::cell::OnceCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[cfg(test)]
pub mod fixture;

/// Главная git-структура, переиспользуемая всеми Phase 5 виджетами.
#[allow(dead_code)]
pub struct GitInfo {
    /// `gix::Repository` — переиспользуется внутренними методами для status/diff.
    pub(crate) repo: gix::Repository,
    pub root_dir: PathBuf,
    pub head: Head,
    pub remotes: HashMap<String, RemoteInfo>,
    /// T3: lazy. None если ещё не считали.
    pub(crate) status_counts: OnceCell<Option<GitStatusCounts>>,
    /// T4: lazy. None если ещё не считали.
    pub(crate) diff_stat: OnceCell<Option<DiffStat>>,
    /// T5: lazy ahead/behind.
    pub(crate) tracking: OnceCell<Option<Tracking>>,
}

#[derive(Debug, Clone, Default)]
pub struct Head {
    /// Имя ветки или None если detached HEAD.
    pub branch: Option<String>,
    /// Полный SHA-1 (40 hex).
    pub sha: Option<String>,
}

impl Head {
    /// Короткий SHA — первые 7 символов (upstream ccstatusline дефолт).
    #[must_use]
    pub fn short_sha(&self) -> Option<String> {
        self.sha.as_ref().map(|s| s.chars().take(7).collect())
    }
}

#[derive(Debug, Clone, Default)]
pub struct GitStatusCounts {
    pub staged: u32,
    pub unstaged: u32,
    pub untracked: u32,
    pub conflicts: u32,
}

impl GitStatusCounts {
    /// Сумма для виджета `GitChanges`: staged + unstaged + untracked + conflicts.
    #[must_use]
    pub const fn total(&self) -> u32 {
        self.staged + self.unstaged + self.untracked + self.conflicts
    }
}

#[derive(Debug, Clone, Default)]
pub struct DiffStat {
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Default)]
pub struct Tracking {
    pub ahead: u32,
    pub behind: u32,
}

#[derive(Debug, Clone)]
pub struct RemoteInfo {
    pub url: String,
    /// Заполняется в T6 (`parse_url`). В T1 — None.
    pub owner: Option<String>,
    pub repo: Option<String>,
}

impl GitInfo {
    /// Поиск репо начиная с `cwd`, вверх до `/` или mount-границы.
    /// Возвращает None если репо не найдено или gix не смог открыть.
    #[must_use]
    pub fn discover(cwd: &Path) -> Option<Self> {
        let repo = gix::discover(cwd).ok()?;
        let root_dir = repo
            .workdir()
            .or_else(|| Some(repo.path()))
            .map(Path::to_path_buf)?;

        let head = parse_head(&repo);
        let remotes = parse_remotes(&repo);

        Some(Self {
            repo,
            root_dir,
            head,
            remotes,
            status_counts: OnceCell::new(),
            diff_stat: OnceCell::new(),
            tracking: OnceCell::new(),
        })
    }

    /// Lazy: считает status один раз, кэширует. None если gix вернул ошибку.
    pub fn status_counts(&self) -> Option<&GitStatusCounts> {
        self.status_counts
            .get_or_init(|| compute_status(&self.repo))
            .as_ref()
    }

    /// Lazy: ahead/behind против upstream. None если нет upstream или detached HEAD.
    pub fn tracking(&self) -> Option<&Tracking> {
        self.tracking
            .get_or_init(|| {
                let cwd = self.repo.workdir()?;
                compute_tracking_shell(cwd)
            })
            .as_ref()
    }

    /// Lazy: diff stat между HEAD и working tree (staged + unstaged). None если
    /// unborn HEAD или `workdir()` недоступен. Вызывается только из
    /// `GitInsertions`/`GitDeletions` — нулевая стоимость без этих виджетов.
    pub fn diff_stat(&self) -> Option<&DiffStat> {
        self.diff_stat
            .get_or_init(|| {
                let work_dir = self.repo.workdir()?;
                compute_diff_stat_shell(work_dir)
            })
            .as_ref()
    }
}

fn compute_tracking_shell(cwd: &std::path::Path) -> Option<Tracking> {
    use std::process::Command;
    let out = Command::new("git")
        .current_dir(cwd)
        .args(["rev-list", "--left-right", "--count", "@{upstream}...HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }
    let behind: u32 = parts[0].parse().ok()?;
    let ahead: u32 = parts[1].parse().ok()?;
    Some(Tracking { ahead, behind })
}

fn compute_diff_stat_shell(cwd: &std::path::Path) -> Option<DiffStat> {
    use std::process::Command;
    let out = Command::new("git")
        .current_dir(cwd)
        .env("LANG", "C")
        .args(["diff", "--shortstat", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let mut stat = DiffStat::default();
    for part in s.split(',') {
        let part = part.trim();
        if let Some(n) = part
            .strip_suffix(" insertions(+)")
            .or_else(|| part.strip_suffix(" insertion(+)"))
        {
            stat.insertions = n.trim().parse().unwrap_or(0);
        } else if let Some(n) = part
            .strip_suffix(" deletions(-)")
            .or_else(|| part.strip_suffix(" deletion(-)"))
        {
            stat.deletions = n.trim().parse().unwrap_or(0);
        }
    }
    Some(stat)
}

fn compute_status(repo: &gix::Repository) -> Option<GitStatusCounts> {
    use gix::status::plumbing::index_as_worktree::EntryStatus;

    let mut counts = GitStatusCounts::default();

    let iter = repo
        .status(gix::progress::Discard)
        .ok()?
        .into_iter(None::<gix::bstr::BString>)
        .ok()?;

    for item in iter.flatten() {
        match item {
            gix::status::Item::TreeIndex(_) => counts.staged += 1,
            gix::status::Item::IndexWorktree(gix::status::index_worktree::Item::Modification {
                status,
                ..
            }) => match status {
                EntryStatus::Change(_) => counts.unstaged += 1,
                EntryStatus::Conflict { .. } => counts.conflicts += 1,
                EntryStatus::NeedsUpdate(_) | EntryStatus::IntentToAdd => {}
            },
            gix::status::Item::IndexWorktree(
                gix::status::index_worktree::Item::DirectoryContents { entry, .. },
            ) => {
                if matches!(entry.status, gix::dir::entry::Status::Untracked) {
                    counts.untracked += 1;
                }
            }
            gix::status::Item::IndexWorktree(gix::status::index_worktree::Item::Rewrite {
                ..
            }) => {
                counts.unstaged += 1;
            }
        }
    }

    Some(counts)
}

fn parse_head(repo: &gix::Repository) -> Head {
    let Ok(head) = repo.head() else {
        return Head::default();
    };
    let branch = head.referent_name().map(|n| {
        let full = n.as_bstr().to_string();
        full.strip_prefix("refs/heads/")
            .map_or(full.clone(), String::from)
    });
    let sha = head.id().map(|id| id.to_string());
    Head { branch, sha }
}

fn parse_remotes(repo: &gix::Repository) -> HashMap<String, RemoteInfo> {
    let mut out = HashMap::new();
    let names = repo.remote_names();
    for name in &names {
        let Ok(remote) = repo.find_remote(name.as_ref()) else {
            continue;
        };
        let Some(url) = remote.url(gix::remote::Direction::Fetch) else {
            continue;
        };
        let url_str = url.to_bstring().to_string();
        out.insert(
            name.to_string(),
            RemoteInfo {
                url: url_str,
                owner: None,
                repo: None,
            },
        );
    }
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::git::fixture::GitFixture;

    #[test]
    fn discover_returns_some_for_git_repo() {
        let f = GitFixture::new();
        let info = GitInfo::discover(f.path()).expect("must discover");
        assert_eq!(
            info.root_dir.canonicalize().unwrap(),
            f.path().canonicalize().unwrap()
        );
    }

    #[test]
    fn discover_returns_none_for_non_repo() {
        let dir = tempfile::tempdir().unwrap();
        assert!(GitInfo::discover(dir.path()).is_none());
    }

    #[test]
    fn head_branch_is_main_after_init() {
        let f = GitFixture::new();
        let info = GitInfo::discover(f.path()).unwrap();
        assert_eq!(info.head.branch.as_deref(), Some("main"));
        assert!(info.head.sha.is_some());
        assert_eq!(info.head.sha.as_ref().unwrap().len(), 40);
    }

    #[test]
    fn short_sha_is_seven_chars() {
        let f = GitFixture::new();
        let info = GitInfo::discover(f.path()).unwrap();
        assert_eq!(info.head.short_sha().unwrap().len(), 7);
    }

    #[test]
    fn detached_head_branch_is_none() {
        let f = GitFixture::new();
        // Создать второй коммит, чтобы было куда отсоединиться.
        f.write_file("a.txt", "1");
        f.git(&["add", "a.txt"]);
        f.commit("c2");
        // Detach.
        f.git(&["checkout", "--detach", "HEAD"]);
        let info = GitInfo::discover(f.path()).unwrap();
        assert!(
            info.head.branch.is_none(),
            "detached HEAD must yield None branch"
        );
        assert!(info.head.sha.is_some());
    }

    #[test]
    fn remotes_parsed_from_config() {
        let f = GitFixture::new();
        f.add_remote("origin", "git@github.com:foo/bar.git");
        f.add_remote("upstream", "https://github.com/baz/bar.git");
        let info = GitInfo::discover(f.path()).unwrap();
        assert_eq!(info.remotes.len(), 2);
        assert_eq!(
            info.remotes.get("origin").unwrap().url,
            "git@github.com:foo/bar.git"
        );
        assert_eq!(
            info.remotes.get("upstream").unwrap().url,
            "https://github.com/baz/bar.git"
        );
        // T6 заполнит owner/repo.
        assert!(info.remotes.get("origin").unwrap().owner.is_none());
    }

    #[test]
    fn status_counts_total_sums_fields() {
        let s = GitStatusCounts {
            staged: 1,
            unstaged: 2,
            untracked: 3,
            conflicts: 4,
        };
        assert_eq!(s.total(), 10);
    }

    #[test]
    fn status_counts_clean_repo() {
        let f = GitFixture::new();
        let info = GitInfo::discover(f.path()).unwrap();
        let s = info.status_counts().expect("must compute");
        assert_eq!(s.total(), 0);
    }

    #[test]
    fn status_counts_untracked_file() {
        let f = GitFixture::new();
        f.write_file("new.txt", "x");
        let info = GitInfo::discover(f.path()).unwrap();
        let s = info.status_counts().unwrap();
        assert_eq!(s.untracked, 1);
        assert_eq!(s.total(), 1);
    }

    #[test]
    fn status_counts_staged_file() {
        let f = GitFixture::new();
        f.write_file("a.txt", "1");
        f.git(&["add", "a.txt"]);
        let info = GitInfo::discover(f.path()).unwrap();
        let s = info.status_counts().unwrap();
        assert_eq!(s.staged, 1);
        assert_eq!(s.untracked, 0);
    }

    #[test]
    fn status_counts_unstaged_modification() {
        let f = GitFixture::new();
        f.write_file("a.txt", "1");
        f.git(&["add", "a.txt"]);
        f.commit("c2");
        f.write_file("a.txt", "2");
        let info = GitInfo::discover(f.path()).unwrap();
        let s = info.status_counts().unwrap();
        assert_eq!(s.unstaged, 1);
        assert_eq!(s.staged, 0);
    }

    #[test]
    fn status_counts_cached_after_first_call() {
        let f = GitFixture::new();
        f.write_file("new.txt", "x");
        let info = GitInfo::discover(f.path()).unwrap();
        let first = info.status_counts().unwrap().total();
        f.write_file("new2.txt", "y");
        let second = info.status_counts().unwrap().total();
        assert_eq!(first, second, "OnceCell must NOT re-compute");
    }

    #[test]
    fn diff_stat_zero_for_clean_repo() {
        let f = GitFixture::new();
        let info = GitInfo::discover(f.path()).unwrap();
        let d = info.diff_stat().expect("must compute");
        assert_eq!(d.insertions, 0);
        assert_eq!(d.deletions, 0);
    }

    #[test]
    fn diff_stat_counts_unstaged_insertions() {
        let f = GitFixture::new();
        f.write_file("a.txt", "line1\nline2\n");
        f.git(&["add", "a.txt"]);
        f.commit("c2");
        f.write_file("a.txt", "line1\nline2\nline3\nline4\n");
        let info = GitInfo::discover(f.path()).unwrap();
        let d = info.diff_stat().unwrap();
        assert_eq!(d.insertions, 2, "expected +2 lines");
        assert_eq!(d.deletions, 0);
    }

    #[test]
    fn diff_stat_counts_unstaged_deletions() {
        let f = GitFixture::new();
        f.write_file("a.txt", "1\n2\n3\n");
        f.git(&["add", "a.txt"]);
        f.commit("c2");
        f.write_file("a.txt", "1\n");
        let info = GitInfo::discover(f.path()).unwrap();
        let d = info.diff_stat().unwrap();
        assert_eq!(d.deletions, 2);
        assert_eq!(d.insertions, 0);
    }

    #[test]
    fn tracking_none_without_upstream() {
        let f = GitFixture::new();
        let info = GitInfo::discover(f.path()).unwrap();
        assert!(info.tracking().is_none());
    }

    #[test]
    fn tracking_zero_when_in_sync() {
        let f = GitFixture::new();
        let bare = tempfile::tempdir().unwrap();
        let bare_path = bare.path().to_str().unwrap();
        f.git(&["init", "--bare", bare_path]);
        f.add_remote("origin", bare_path);
        f.git(&["push", "-u", "origin", "main"]);

        let info = GitInfo::discover(f.path()).unwrap();
        let t = info.tracking().expect("must have tracking");
        assert_eq!(t.ahead, 0);
        assert_eq!(t.behind, 0);
    }

    #[test]
    fn tracking_ahead_after_local_commit() {
        let f = GitFixture::new();
        let bare = tempfile::tempdir().unwrap();
        let bare_path = bare.path().to_str().unwrap();
        f.git(&["init", "--bare", bare_path]);
        f.add_remote("origin", bare_path);
        f.git(&["push", "-u", "origin", "main"]);

        f.write_file("a.txt", "x");
        f.git(&["add", "a.txt"]);
        f.commit("c2");

        let info = GitInfo::discover(f.path()).unwrap();
        let t = info.tracking().unwrap();
        assert_eq!(t.ahead, 1);
        assert_eq!(t.behind, 0);
    }

    #[test]
    fn diff_stat_cached() {
        let f = GitFixture::new();
        f.write_file("a.txt", "1\n");
        f.git(&["add", "a.txt"]);
        f.commit("c2");
        f.write_file("a.txt", "1\n2\n");
        let info = GitInfo::discover(f.path()).unwrap();
        let first = info.diff_stat().unwrap().insertions;
        f.write_file("a.txt", "1\n2\n3\n");
        let second = info.diff_stat().unwrap().insertions;
        assert_eq!(first, second, "OnceCell must NOT re-compute");
    }
}
