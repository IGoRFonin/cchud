//! Environment widgets.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::path::Path;

use crate::types::config::CurrentWorkingDirParams;
use crate::widgets::{RenderContext, Widget};

pub struct ClaudeAccountEmail;
pub struct FreeMemory;
pub struct CurrentWorkingDir {
    pub params: CurrentWorkingDirParams,
}

impl Widget for ClaudeAccountEmail {
    fn id(&self) -> &'static str {
        "claude-account-email"
    }
    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        crate::commands::env_loader::claude_account_email().map(str::to_string)
    }
}

impl Widget for FreeMemory {
    fn id(&self) -> &'static str {
        "free-memory"
    }
    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();
        let bytes = sys.available_memory();
        Some(format!("💾 {}", crate::util::format_memory::format(bytes)))
    }
}

impl Widget for CurrentWorkingDir {
    fn id(&self) -> &'static str {
        "current-working-dir"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let cwd = ctx.payload.cwd.as_deref()?;
        let home = dirs::home_dir();
        let rendered = render_cwd(cwd, home.as_deref(), &self.params);
        match self.params.prefix.as_deref() {
            Some(prefix) => Some(format!("{prefix}{rendered}")),
            None => Some(rendered),
        }
    }
}

fn render_cwd(cwd: &str, home: Option<&Path>, params: &CurrentWorkingDirParams) -> String {
    if params.fish_style {
        return abbreviate_path(cwd, home);
    }
    let with_home = if params.abbreviate_home {
        replace_home(cwd, home)
    } else {
        cwd.to_string()
    };
    match params.segments {
        Some(n) if n > 0 => take_last_segments(&with_home, n, params.abbreviate_home),
        _ => with_home,
    }
}

fn home_str(home: Option<&Path>) -> Option<String> {
    home.and_then(|p| p.to_str().map(str::to_string))
}

fn replace_home(path: &str, home: Option<&Path>) -> String {
    match home_str(home) {
        Some(h) if path == h => "~".to_string(),
        Some(h) if path.starts_with(&format!("{h}/")) => format!("~{}", &path[h.len()..]),
        _ => path.to_string(),
    }
}

fn take_last_segments(path: &str, n: u32, abbreviate_home: bool) -> String {
    let segs: Vec<&str> = path.split(['/', '\\']).filter(|s| !s.is_empty()).collect();
    let n = n as usize;
    if segs.len() <= n {
        return path.to_string();
    }
    let tail = segs[segs.len() - n..].join("/");
    let prefix = if abbreviate_home && path.starts_with('~') {
        "~/.../"
    } else {
        ".../"
    };
    format!("{prefix}{tail}")
}

fn abbreviate_path(cwd: &str, home: Option<&Path>) -> String {
    let with_home = replace_home(cwd, home);
    let starts_with_slash = with_home.starts_with('/');
    let segs: Vec<&str> = with_home
        .split(['/', '\\'])
        .filter(|s| !s.is_empty())
        .collect();
    if segs.is_empty() {
        return with_home;
    }
    let last = segs.len() - 1;
    let mut parts: Vec<String> = Vec::with_capacity(segs.len());
    for (i, seg) in segs.iter().enumerate() {
        if i == 0 || i == last {
            parts.push((*seg).to_string());
        } else {
            parts.push(abbreviate_segment(seg));
        }
    }
    let joined = parts.join("/");
    if starts_with_slash {
        format!("/{joined}")
    } else {
        joined
    }
}

fn abbreviate_segment(seg: &str) -> String {
    let mut chars = seg.chars();
    match chars.next() {
        None => String::new(),
        Some('.') => chars
            .next()
            .map_or_else(|| ".".to_string(), |c| format!(".{c}")),
        Some(c) => c.to_string(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::widgets::test_helpers::payload_no_transcript;

    #[test]
    fn free_memory_returns_some_string() {
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = RenderContext::new(&p, &s);
        let out = FreeMemory
            .render(&ctx)
            .expect("system memory always present");
        assert!(out.starts_with("💾 "), "got: {out}");
        assert!(out.chars().last().is_some_and(|c| "bkMGT".contains(c)));
    }

    #[test]
    fn free_memory_render_second_call_does_not_panic() {
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = RenderContext::new(&p, &s);
        let _ = FreeMemory.render(&ctx);
    }

    #[test]
    fn claude_account_email_render_with_lock_prefilled() {
        use crate::commands::env_loader::{ClaudeJson, OauthAccount, set_claude_json_for_tests};
        // OnceLock is first-writer-wins; set_claude_json_for_tests is a no-op if already set.
        // We pre-fill then call render() — the result may be Some(email) or None depending on
        // which test in this binary won the race, both are valid outcomes.
        set_claude_json_for_tests(Some(ClaudeJson {
            oauth_account: Some(OauthAccount {
                email_address: Some("widget_test@example.com".into()),
            }),
        }));
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = RenderContext::new(&p, &s);
        let out = ClaudeAccountEmail.render(&ctx);
        if let Some(email) = out {
            assert!(
                email.contains('@'),
                "render() returned non-email string: {email}"
            );
        }
    }

    #[test]
    fn cwd_full_path_when_no_options_set() {
        let params = CurrentWorkingDirParams::default();
        let out = render_cwd("/Users/me/proj/cchud", Some(Path::new("/Users/other")), &params);
        assert_eq!(out, "/Users/me/proj/cchud");
    }

    #[test]
    fn cwd_abbreviate_home_replaces_with_tilde() {
        let params = CurrentWorkingDirParams {
            abbreviate_home: true,
            ..Default::default()
        };
        let out = render_cwd("/Users/me/proj", Some(Path::new("/Users/me")), &params);
        assert_eq!(out, "~/proj");
    }

    #[test]
    fn cwd_segments_takes_last_n_with_ellipsis() {
        let params = CurrentWorkingDirParams {
            segments: Some(2),
            ..Default::default()
        };
        let out = render_cwd("/Users/me/proj/cchud", None, &params);
        assert_eq!(out, ".../proj/cchud");
    }

    #[test]
    fn cwd_segments_combined_with_abbreviate_home() {
        let params = CurrentWorkingDirParams {
            segments: Some(2),
            abbreviate_home: true,
            ..Default::default()
        };
        let out = render_cwd("/Users/me/proj/cchud", Some(Path::new("/Users/me")), &params);
        assert_eq!(out, "~/.../proj/cchud");
    }

    #[test]
    fn cwd_segments_no_op_when_path_short() {
        let params = CurrentWorkingDirParams {
            segments: Some(10),
            ..Default::default()
        };
        let out = render_cwd("/Users/me", None, &params);
        assert_eq!(out, "/Users/me");
    }

    #[test]
    fn cwd_fish_style_compresses_intermediate_segments() {
        let params = CurrentWorkingDirParams {
            fish_style: true,
            ..Default::default()
        };
        let out = render_cwd(
            "/Users/me/projects/cchud",
            Some(Path::new("/nope")),
            &params,
        );
        assert_eq!(out, "/Users/m/p/cchud");
    }

    #[test]
    fn cwd_fish_style_preserves_hidden_two_chars() {
        let params = CurrentWorkingDirParams {
            fish_style: true,
            ..Default::default()
        };
        let out = render_cwd("/.config/foo/bar/baz", None, &params);
        // First seg ".config" whole; "foo" → "f"; "bar" → "b"; last whole.
        assert_eq!(out, "/.config/f/b/baz");
    }

    #[test]
    fn cwd_fish_style_replaces_home_with_tilde() {
        let params = CurrentWorkingDirParams {
            fish_style: true,
            ..Default::default()
        };
        let out = render_cwd("/Users/me/projects/cchud", Some(Path::new("/Users/me")), &params);
        assert_eq!(out, "~/p/cchud");
    }

    #[test]
    fn claude_account_email_render_output_format() {
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = RenderContext::new(&p, &s);
        let out = ClaudeAccountEmail.render(&ctx);
        if let Some(email) = out {
            assert!(
                email.contains('@'),
                "render() returned non-email string: {email}"
            );
        }
    }
}
