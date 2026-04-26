//! `CustomCommand` widget — Phase 3 Task 7.
//!
//! Spawns a user-configured subprocess argv-style (без shell), читает
//! stdout с timeout'ом и возвращает trimmed-content. Любой негативный
//! сценарий → None молча (Phase 3 не диагностирует stderr).
//!
//! Security:
//! - Без `sh -c` → нет shell-injection.
//! - Inherit parent env → пользователь сам отвечает за безопасность
//!   команды; в README предупреждение про API-keys.
//! - Stdin = null → child не ждёт input.
//! - Stderr = null → не загрязняем stderr cchud.
//!
//! Платформа: Unix-only тесты под `#[cfg(unix)]`. Windows валиден на
//! компиляции, но behavioural-тесты (echo / sleep / false) отложены до
//! Phase 9 (distribution).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::Duration;

use wait_timeout::ChildExt;

use crate::types::config::CustomCommandParams;
use crate::widgets::{RenderContext, Widget};

pub struct CustomCommand {
    pub params: CustomCommandParams,
}

impl Widget for CustomCommand {
    fn id(&self) -> &'static str {
        "CustomCommand"
    }

    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        let mut child = Command::new(&self.params.command)
            .args(&self.params.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;

        let timeout = Duration::from_millis(self.params.timeout_ms);
        match child.wait_timeout(timeout).ok()? {
            None => {
                let _ = child.kill();
                let _ = child.wait();
                None
            }
            Some(status) => {
                if !status.success() {
                    return None;
                }
                let mut out = String::new();
                child.stdout.as_mut()?.read_to_string(&mut out).ok()?;
                let trimmed = out.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            }
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::config::default_line;
    use crate::types::payload::{ModelInfo, StatusPayload, Workspace};

    fn empty_payload() -> StatusPayload {
        StatusPayload {
            session_id: "test".into(),
            model: ModelInfo {
                id: "claude-sonnet-4-6".into(),
                display_name: "Sonnet 4.6".into(),
            },
            workspace: Workspace {
                current_dir: "/tmp".into(),
                project_dir: None,
                added_dirs: None,
            },
            transcript_path: None,
            cwd: None,
            version: None,
            fast_mode: None,
            exceeds_200k_tokens: None,
            output_style: None,
            cost: None,
            context_window: None,
            worktree: None,
            vim: None,
            rate_limits: None,
            effort: None,
            thinking: None,
        }
    }

    fn ctx_with<'a>(
        p: &'a StatusPayload,
        s: &'a crate::types::config::Settings,
    ) -> RenderContext<'a> {
        RenderContext::new(p, s)
    }

    #[test]
    fn echo_returns_trimmed_stdout() {
        let p = empty_payload();
        let s = default_line();
        let w = CustomCommand {
            params: CustomCommandParams {
                command: "echo".into(),
                args: vec!["hello-cc".into()],
                timeout_ms: 1_000,
            },
        };
        assert_eq!(w.render(&ctx_with(&p, &s)), Some("hello-cc".into()));
    }

    #[test]
    fn echo_empty_returns_none() {
        let p = empty_payload();
        let s = default_line();
        let w = CustomCommand {
            params: CustomCommandParams {
                command: "true".into(),
                args: vec![],
                timeout_ms: 1_000,
            },
        };
        assert_eq!(w.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn nonzero_exit_returns_none() {
        let p = empty_payload();
        let s = default_line();
        let w = CustomCommand {
            params: CustomCommandParams {
                command: "false".into(),
                args: vec![],
                timeout_ms: 1_000,
            },
        };
        assert_eq!(w.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn missing_binary_returns_none() {
        let p = empty_payload();
        let s = default_line();
        let w = CustomCommand {
            params: CustomCommandParams {
                command: "/no/such/binary-xyz-12345".into(),
                args: vec![],
                timeout_ms: 1_000,
            },
        };
        assert_eq!(w.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn timeout_kills_child_and_returns_none() {
        let p = empty_payload();
        let s = default_line();
        let w = CustomCommand {
            params: CustomCommandParams {
                command: "sleep".into(),
                args: vec!["5".into()],
                timeout_ms: 50,
            },
        };
        let start = std::time::Instant::now();
        let out = w.render(&ctx_with(&p, &s));
        let elapsed = start.elapsed();
        assert_eq!(out, None);
        assert!(
            elapsed < Duration::from_millis(1_000),
            "render did not respect timeout: elapsed = {elapsed:?}"
        );
    }

    #[test]
    fn trims_trailing_newline_and_whitespace() {
        let p = empty_payload();
        let s = default_line();
        let w = CustomCommand {
            params: CustomCommandParams {
                command: "printf".into(),
                args: vec!["  spaced  \n".into()],
                timeout_ms: 1_000,
            },
        };
        assert_eq!(w.render(&ctx_with(&p, &s)), Some("spaced".into()));
    }

    #[test]
    fn argv_avoids_shell_interpretation() {
        let p = empty_payload();
        let s = default_line();
        let w = CustomCommand {
            params: CustomCommandParams {
                command: "echo".into(),
                args: vec!["$(whoami); echo INJECTED".into()],
                timeout_ms: 1_000,
            },
        };
        let out = w.render(&ctx_with(&p, &s)).unwrap();
        assert_eq!(out, "$(whoami); echo INJECTED");
    }
}
