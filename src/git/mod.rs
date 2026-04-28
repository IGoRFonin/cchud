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

use std::cell::OnceCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[cfg(test)]
pub mod fixture;
pub mod pr;
pub mod remote;

/// Главная git-структура, переиспользуемая всеми Phase 5 виджетами.
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
    #[allow(dead_code)]
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
            .get_or_init(|| compute_tracking_gix(&self.repo))
            .as_ref()
    }

    /// Lazy: diff stat между HEAD и working tree (staged + unstaged). None если
    /// unborn HEAD или `workdir()` недоступен. Вызывается только из
    /// `GitInsertions`/`GitDeletions` — нулевая стоимость без этих виджетов.
    pub fn diff_stat(&self) -> Option<&DiffStat> {
        self.diff_stat
            .get_or_init(|| compute_diff_stat_gix(&self.repo))
            .as_ref()
    }
}

fn compute_tracking_gix(repo: &gix::Repository) -> Option<Tracking> {
    use gix::bstr::ByteSlice;

    let head = repo.head().ok()?;
    let full_name = head.referent_name()?;
    let full_name_str = full_name.as_bstr().to_str().ok()?;
    let branch_short = full_name_str.strip_prefix("refs/heads/")?;

    let config = repo.config_snapshot();
    let remote_key = format!("branch.{branch_short}.remote");
    let merge_key = format!("branch.{branch_short}.merge");

    let remote: String = config
        .string(remote_key.as_str())?
        .to_str()
        .ok()?
        .to_owned();
    let merge: String = config.string(merge_key.as_str())?.to_str().ok()?.to_owned();
    let upstream_branch = merge.strip_prefix("refs/heads/").unwrap_or(&merge);
    let upstream_ref = format!("refs/remotes/{remote}/{upstream_branch}");

    let local_id = head.id()?.detach();
    let upstream_id = repo
        .find_reference(upstream_ref.as_str())
        .ok()?
        .peel_to_id()
        .ok()?
        .detach();

    // Commits reachable from local but not upstream = ahead.
    let ahead: u32 = repo
        .rev_walk(std::iter::once(local_id))
        .with_hidden(std::iter::once(upstream_id))
        .all()
        .ok()?
        .filter_map(Result::ok)
        .count()
        .try_into()
        .unwrap_or(u32::MAX);

    // Commits reachable from upstream but not local = behind.
    let behind: u32 = repo
        .rev_walk(std::iter::once(upstream_id))
        .with_hidden(std::iter::once(local_id))
        .all()
        .ok()?
        .filter_map(Result::ok)
        .count()
        .try_into()
        .unwrap_or(u32::MAX);

    Some(Tracking { ahead, behind })
}

/// Compute insertions/deletions from HEAD to current workdir (staged + unstaged).
/// Equivalent to `git diff --shortstat HEAD` without a subprocess.
fn compute_diff_stat_gix(repo: &gix::Repository) -> Option<DiffStat> {
    let work_dir = repo.workdir()?.to_owned();
    let mut stat = DiffStat::default();

    let iter = repo
        .status(gix::progress::Discard)
        .ok()?
        .into_iter(None::<gix::bstr::BString>)
        .ok()?;

    for item in iter.flatten() {
        match item {
            // Staged change: HEAD tree blob → index blob.
            gix::status::Item::TreeIndex(ref change) => {
                use gix::diff::index::Change;
                let (old_id, new_id) = match change {
                    Change::Addition { id, .. } => (None, Some(id.as_ref())),
                    Change::Deletion { id, .. } => (Some(id.as_ref()), None),
                    Change::Modification {
                        previous_id, id, ..
                    } => (Some(previous_id.as_ref()), Some(id.as_ref())),
                    Change::Rewrite { .. } => continue,
                };
                let old = old_id
                    .and_then(|oid| repo.find_blob(oid).ok())
                    .map(|b| b.data.clone());
                let new = new_id
                    .and_then(|oid| repo.find_blob(oid).ok())
                    .map(|b| b.data.clone());
                accumulate_diff(&mut stat, old.as_deref(), new.as_deref());
            }
            // Unstaged change: index blob → disk content.
            gix::status::Item::IndexWorktree(gix::status::index_worktree::Item::Modification {
                ref entry,
                ref rela_path,
                ..
            }) => {
                let old = repo.find_blob(entry.id).ok().map(|b| b.data.clone());
                let new = {
                    use gix::bstr::ByteSlice;
                    rela_path
                        .to_path()
                        .ok()
                        .and_then(|p| std::fs::read(work_dir.join(p)).ok())
                };
                accumulate_diff(&mut stat, old.as_deref(), new.as_deref());
            }
            gix::status::Item::IndexWorktree(_) => {}
        }
    }

    Some(stat)
}

fn accumulate_diff(stat: &mut DiffStat, old: Option<&[u8]>, new: Option<&[u8]>) {
    match (old, new) {
        (None, Some(n)) => stat.insertions += lines_in(n),
        (Some(o), None) => stat.deletions += lines_in(o),
        (Some(o), Some(n)) => {
            let (ins, del) = blob_line_diff(o, n);
            stat.insertions += ins;
            stat.deletions += del;
        }
        (None, None) => {}
    }
}

fn lines_in(data: &[u8]) -> u32 {
    if data.is_empty() {
        return 0;
    }
    let newlines: u32 = data
        .iter()
        .copied()
        .filter(|&b| b == b'\n')
        .count()
        .try_into()
        .unwrap_or(u32::MAX);
    newlines + u32::from(!data.ends_with(b"\n"))
}

fn blob_line_diff(old: &[u8], new: &[u8]) -> (u32, u32) {
    use gix::diff::blob::{Algorithm, diff, intern::InternedInput, sink::Counter};
    let input = InternedInput::new(old, new);
    let counter = diff(Algorithm::Histogram, &input, Counter::default());
    (counter.insertions, counter.removals)
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
        let (owner, repo) =
            remote::parse_url(&url_str).map_or((None, None), |(o, r)| (Some(o), Some(r)));
        out.insert(
            name.to_string(),
            RemoteInfo {
                url: url_str,
                owner,
                repo,
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
        // T6 заполняет owner/repo через parse_url.
        assert_eq!(
            info.remotes.get("origin").unwrap().owner.as_deref(),
            Some("foo")
        );
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
