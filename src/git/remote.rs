//! Remote URL parser.
//!
//! Hand-parser без зависимости `regex` (экономия ~300 КБ).
//! Покрывает 4 канонических формата git remote URL.

#![deny(clippy::unwrap_used, clippy::expect_used)]

/// Парсит remote URL в `(owner, repo)`. None если формат не распознан.
///
/// Поддерживаемые форматы:
/// - `git@host:owner/repo.git` (SSH short)
/// - `ssh://git@host/owner/repo.git` (SSH explicit)
/// - `https://host/owner/repo.git` (HTTPS)
/// - `https://host/owner/repo` (HTTPS no .git)
#[must_use]
pub fn parse_url(url: &str) -> Option<(String, String)> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }

    // 1. SSH explicit: ssh://[user@]host/path
    if let Some(rest) = url.strip_prefix("ssh://") {
        let after_host = strip_user_at_host_slash(rest)?;
        return parse_owner_repo(after_host);
    }

    // 2. HTTPS / HTTP
    for prefix in ["https://", "http://"] {
        if let Some(rest) = url.strip_prefix(prefix) {
            // rest = host/path...
            let (_, path) = rest.split_once('/')?;
            return parse_owner_repo(path);
        }
    }

    // 3. SSH short: user@host:path
    if let Some((_, path)) = url.split_once(':') {
        // Защита от случая 'C:\path' на Windows: путь после ':' должен начинаться
        // с буквы и содержать '/'.
        if !path.is_empty() && !path.starts_with('/') && path.contains('/') {
            return parse_owner_repo(path);
        }
    }

    None
}

fn strip_user_at_host_slash(s: &str) -> Option<&str> {
    // ssh://...  rest = [user@]host/path
    let after_at = s.split_once('@').map_or(s, |(_, r)| r);
    after_at.split_once('/').map(|(_, path)| path)
}

fn parse_owner_repo(path: &str) -> Option<(String, String)> {
    let path = path.trim_end_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let (owner, repo) = path.rsplit_once('/')?;
    if repo.is_empty() || owner.is_empty() {
        return None;
    }
    // Group в gitlab может быть `group/subgroup/repo`. Берём *последние* два сегмента.
    // owner = последний сегмент перед repo (не вся group-цепочка, чтобы соответствовать
    // ccstatusline upstream поведению — он берёт `group/repo` или `subgroup/repo`).
    let owner = owner.rsplit('/').next().unwrap_or(owner);
    Some((owner.to_string(), repo.to_string()))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn parses_ssh_short() {
        assert_eq!(
            parse_url("git@github.com:foo/bar.git"),
            Some(("foo".into(), "bar".into()))
        );
    }

    #[test]
    fn parses_ssh_explicit() {
        assert_eq!(
            parse_url("ssh://git@gitlab.com/foo/bar.git"),
            Some(("foo".into(), "bar".into()))
        );
    }

    #[test]
    fn parses_https_with_dot_git() {
        assert_eq!(
            parse_url("https://github.com/foo/bar.git"),
            Some(("foo".into(), "bar".into()))
        );
    }

    #[test]
    fn parses_https_without_dot_git() {
        assert_eq!(
            parse_url("https://github.com/foo/bar"),
            Some(("foo".into(), "bar".into()))
        );
    }

    #[test]
    fn parses_https_with_trailing_slash() {
        assert_eq!(
            parse_url("https://github.com/foo/bar/"),
            Some(("foo".into(), "bar".into()))
        );
    }

    #[test]
    fn parses_gitlab_subgroup_takes_immediate_owner() {
        // `group/subgroup/repo` — owner = `subgroup`, repo = `repo`
        assert_eq!(
            parse_url("https://gitlab.com/group/subgroup/repo.git"),
            Some(("subgroup".into(), "repo".into()))
        );
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(parse_url(""), None);
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(parse_url("not a url"), None);
        assert_eq!(parse_url("ftp://example.com/foo/bar"), None);
    }

    #[test]
    fn rejects_windows_path_lookalike() {
        // 'C:\foo\bar' — после ':' нет '/', не воспримем как SSH.
        assert_eq!(parse_url("C:foo\\bar"), None);
    }

    #[test]
    fn handles_dashes_and_dots_in_repo_name() {
        assert_eq!(
            parse_url("git@github.com:foo-bar/repo.with.dots.git"),
            Some(("foo-bar".into(), "repo.with.dots".into()))
        );
    }
}
