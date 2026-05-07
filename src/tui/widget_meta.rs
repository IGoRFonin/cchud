//! Static widget palette registry.
//!
//! 61 виджет из `WidgetConfig` (без `AlignRight` — sentinel, не показывается в палитре).
//! Категории отвечают «бакету» в UI (Model / Static / Trivial / Session / Context / ...).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::types::config::{
    ContextBarParams, CurrentWorkingDirParams, CustomCommandParams, CustomSymbolParams,
    CustomTextParams, LinkParams, ModelParams, WidgetConfig,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetCategory {
    Model,
    Static,
    Trivial,
    Session,
    Context,
    Worktree,
    Git,
    Transcript,
    Usage,
    Env,
}

impl WidgetCategory {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Model => "Model",
            Self::Static => "Static",
            Self::Trivial => "Trivial",
            Self::Session => "Session",
            Self::Context => "Context",
            Self::Worktree => "Worktree",
            Self::Git => "Git",
            Self::Transcript => "Transcript",
            Self::Usage => "Usage",
            Self::Env => "Env",
        }
    }
}

pub struct WidgetMeta {
    pub display: &'static str,
    pub kebab_type: &'static str,
    pub category: WidgetCategory,
    pub factory: fn() -> WidgetConfig,
}

/// 61 widgets — match `WidgetConfig` enum minus `AlignRight`. Order реализует
/// natural grouping: каждая категория contiguous → palette по группам.
#[allow(clippy::too_many_lines)]
pub static ALL_KINDS: &[WidgetMeta] = &[
    // Model (1)
    WidgetMeta {
        display: "Model",
        kebab_type: "model",
        category: WidgetCategory::Model,
        factory: || WidgetConfig::Model {
            params: ModelParams {},
        },
    },
    // Static (5)
    WidgetMeta {
        display: "Separator",
        kebab_type: "separator",
        category: WidgetCategory::Static,
        factory: || WidgetConfig::Separator,
    },
    WidgetMeta {
        display: "Custom Text",
        kebab_type: "custom-text",
        category: WidgetCategory::Static,
        factory: || WidgetConfig::CustomText {
            params: CustomTextParams {
                text: "text".into(),
            },
        },
    },
    WidgetMeta {
        display: "Custom Symbol",
        kebab_type: "custom-symbol",
        category: WidgetCategory::Static,
        factory: || WidgetConfig::CustomSymbol {
            params: CustomSymbolParams {
                symbol: "★".into()
            },
        },
    },
    WidgetMeta {
        display: "Link",
        kebab_type: "link",
        category: WidgetCategory::Static,
        factory: || WidgetConfig::Link {
            params: LinkParams {
                url: "https://example.com".into(),
                label: None,
            },
        },
    },
    WidgetMeta {
        display: "Custom Command",
        kebab_type: "custom-command",
        category: WidgetCategory::Static,
        factory: || WidgetConfig::CustomCommand {
            params: CustomCommandParams {
                command: "echo".into(),
                args: vec!["hi".into()],
                timeout_ms: 200,
            },
        },
    },
    // Trivial (5)
    WidgetMeta {
        display: "Version",
        kebab_type: "version",
        category: WidgetCategory::Trivial,
        factory: || WidgetConfig::Version,
    },
    WidgetMeta {
        display: "Claude Session Id",
        kebab_type: "claude-session-id",
        category: WidgetCategory::Trivial,
        factory: || WidgetConfig::ClaudeSessionId,
    },
    WidgetMeta {
        display: "Terminal Width",
        kebab_type: "terminal-width",
        category: WidgetCategory::Trivial,
        factory: || WidgetConfig::TerminalWidth,
    },
    WidgetMeta {
        display: "Output Style",
        kebab_type: "output-style",
        category: WidgetCategory::Trivial,
        factory: || WidgetConfig::OutputStyle,
    },
    WidgetMeta {
        display: "Vim Mode",
        kebab_type: "vim-mode",
        category: WidgetCategory::Trivial,
        factory: || WidgetConfig::VimMode,
    },
    // Session (3)
    WidgetMeta {
        display: "Session Name",
        kebab_type: "session-name",
        category: WidgetCategory::Session,
        factory: || WidgetConfig::SessionName,
    },
    WidgetMeta {
        display: "Session Clock",
        kebab_type: "session-clock",
        category: WidgetCategory::Session,
        factory: || WidgetConfig::SessionClock,
    },
    WidgetMeta {
        display: "Session Cost",
        kebab_type: "session-cost",
        category: WidgetCategory::Session,
        factory: || WidgetConfig::SessionCost,
    },
    // Context (6)
    WidgetMeta {
        display: "Context Length",
        kebab_type: "context-length",
        category: WidgetCategory::Context,
        factory: || WidgetConfig::ContextLength,
    },
    WidgetMeta {
        display: "Context %",
        kebab_type: "context-percentage",
        category: WidgetCategory::Context,
        factory: || WidgetConfig::ContextPercentage,
    },
    WidgetMeta {
        display: "Context % Usable",
        kebab_type: "context-percentage-usable",
        category: WidgetCategory::Context,
        factory: || WidgetConfig::ContextPercentageUsable,
    },
    WidgetMeta {
        display: "Context Bar",
        kebab_type: "context-bar",
        category: WidgetCategory::Context,
        factory: || WidgetConfig::ContextBar {
            params: ContextBarParams { width: 10 },
        },
    },
    WidgetMeta {
        display: "Tokens Input",
        kebab_type: "tokens-input",
        category: WidgetCategory::Context,
        factory: || WidgetConfig::TokensInput,
    },
    WidgetMeta {
        display: "Tokens Output",
        kebab_type: "tokens-output",
        category: WidgetCategory::Context,
        factory: || WidgetConfig::TokensOutput,
    },
    // Worktree (5)
    WidgetMeta {
        display: "Worktree",
        kebab_type: "worktree",
        category: WidgetCategory::Worktree,
        factory: || WidgetConfig::Worktree,
    },
    WidgetMeta {
        display: "Worktree Mode",
        kebab_type: "worktree-mode",
        category: WidgetCategory::Worktree,
        factory: || WidgetConfig::WorktreeMode,
    },
    WidgetMeta {
        display: "Worktree Name",
        kebab_type: "worktree-name",
        category: WidgetCategory::Worktree,
        factory: || WidgetConfig::WorktreeName,
    },
    WidgetMeta {
        display: "Worktree Branch",
        kebab_type: "worktree-branch",
        category: WidgetCategory::Worktree,
        factory: || WidgetConfig::WorktreeBranch,
    },
    WidgetMeta {
        display: "Worktree Original Branch",
        kebab_type: "worktree-original-branch",
        category: WidgetCategory::Worktree,
        factory: || WidgetConfig::WorktreeOriginalBranch,
    },
    // Git (20)
    WidgetMeta {
        display: "Git Branch",
        kebab_type: "git-branch",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitBranch,
    },
    WidgetMeta {
        display: "Git SHA",
        kebab_type: "git-sha",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitSha,
    },
    WidgetMeta {
        display: "Git Root Dir",
        kebab_type: "git-root-dir",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitRootDir,
    },
    WidgetMeta {
        display: "Git Status",
        kebab_type: "git-status",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitStatus,
    },
    WidgetMeta {
        display: "Git Changes",
        kebab_type: "git-changes",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitChanges,
    },
    WidgetMeta {
        display: "Git Staged",
        kebab_type: "git-staged",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitStaged,
    },
    WidgetMeta {
        display: "Git Unstaged",
        kebab_type: "git-unstaged",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitUnstaged,
    },
    WidgetMeta {
        display: "Git Untracked",
        kebab_type: "git-untracked",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitUntracked,
    },
    WidgetMeta {
        display: "Git Conflicts",
        kebab_type: "git-conflicts",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitConflicts,
    },
    WidgetMeta {
        display: "Git Insertions",
        kebab_type: "git-insertions",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitInsertions,
    },
    WidgetMeta {
        display: "Git Deletions",
        kebab_type: "git-deletions",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitDeletions,
    },
    WidgetMeta {
        display: "Git Ahead/Behind",
        kebab_type: "git-ahead-behind",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitAheadBehind,
    },
    WidgetMeta {
        display: "Git Origin Owner",
        kebab_type: "git-origin-owner",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitOriginOwner,
    },
    WidgetMeta {
        display: "Git Origin Repo",
        kebab_type: "git-origin-repo",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitOriginRepo,
    },
    WidgetMeta {
        display: "Git Origin Owner/Repo",
        kebab_type: "git-origin-owner-repo",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitOriginOwnerRepo,
    },
    WidgetMeta {
        display: "Git Upstream Owner",
        kebab_type: "git-upstream-owner",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitUpstreamOwner,
    },
    WidgetMeta {
        display: "Git Upstream Repo",
        kebab_type: "git-upstream-repo",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitUpstreamRepo,
    },
    WidgetMeta {
        display: "Git Upstream Owner/Repo",
        kebab_type: "git-upstream-owner-repo",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitUpstreamOwnerRepo,
    },
    WidgetMeta {
        display: "Git Is Fork",
        kebab_type: "git-is-fork",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitIsFork,
    },
    WidgetMeta {
        display: "Git PR",
        kebab_type: "git-pr",
        category: WidgetCategory::Git,
        factory: || WidgetConfig::GitPr,
    },
    // Transcript (9)
    WidgetMeta {
        display: "Tokens Cached",
        kebab_type: "tokens-cached",
        category: WidgetCategory::Transcript,
        factory: || WidgetConfig::TokensCached,
    },
    WidgetMeta {
        display: "Tokens Total",
        kebab_type: "tokens-total",
        category: WidgetCategory::Transcript,
        factory: || WidgetConfig::TokensTotal,
    },
    WidgetMeta {
        display: "Input Speed",
        kebab_type: "input-speed",
        category: WidgetCategory::Transcript,
        factory: || WidgetConfig::InputSpeed,
    },
    WidgetMeta {
        display: "Output Speed",
        kebab_type: "output-speed",
        category: WidgetCategory::Transcript,
        factory: || WidgetConfig::OutputSpeed,
    },
    WidgetMeta {
        display: "Total Speed",
        kebab_type: "total-speed",
        category: WidgetCategory::Transcript,
        factory: || WidgetConfig::TotalSpeed,
    },
    WidgetMeta {
        display: "Block Timer",
        kebab_type: "block-timer",
        category: WidgetCategory::Transcript,
        factory: || WidgetConfig::BlockTimer,
    },
    WidgetMeta {
        display: "Session Duration",
        kebab_type: "session-duration",
        category: WidgetCategory::Transcript,
        factory: || WidgetConfig::SessionDuration,
    },
    WidgetMeta {
        display: "Thinking Effort",
        kebab_type: "thinking-effort",
        category: WidgetCategory::Transcript,
        factory: || WidgetConfig::ThinkingEffort,
    },
    WidgetMeta {
        display: "Skills",
        kebab_type: "skills",
        category: WidgetCategory::Transcript,
        factory: || WidgetConfig::Skills,
    },
    // Usage (4)
    WidgetMeta {
        display: "Session Usage",
        kebab_type: "session-usage",
        category: WidgetCategory::Usage,
        factory: || WidgetConfig::SessionUsage,
    },
    WidgetMeta {
        display: "Weekly Usage",
        kebab_type: "weekly-usage",
        category: WidgetCategory::Usage,
        factory: || WidgetConfig::WeeklyUsage,
    },
    WidgetMeta {
        display: "Block Reset Timer",
        kebab_type: "block-reset-timer",
        category: WidgetCategory::Usage,
        factory: || WidgetConfig::BlockResetTimer,
    },
    WidgetMeta {
        display: "Weekly Reset Timer",
        kebab_type: "weekly-reset-timer",
        category: WidgetCategory::Usage,
        factory: || WidgetConfig::WeeklyResetTimer,
    },
    // Env (3)
    WidgetMeta {
        display: "Claude Account Email",
        kebab_type: "claude-account-email",
        category: WidgetCategory::Env,
        factory: || WidgetConfig::ClaudeAccountEmail,
    },
    WidgetMeta {
        display: "Free Memory",
        kebab_type: "free-memory",
        category: WidgetCategory::Env,
        factory: || WidgetConfig::FreeMemory,
    },
    WidgetMeta {
        display: "Current Working Dir",
        kebab_type: "current-working-dir",
        category: WidgetCategory::Env,
        factory: || WidgetConfig::CurrentWorkingDir {
            params: CurrentWorkingDirParams::default(),
        },
    },
];

/// Lookup by kebab `type` field. Used by `commands::import::parse_best_effort`
/// (whitelist) и палитрой при поиске по строгому имени.
#[must_use]
pub fn lookup_by_kebab(kebab: &str) -> Option<&'static WidgetMeta> {
    ALL_KINDS.iter().find(|m| m.kebab_type == kebab)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn all_kinds_has_61_entries() {
        assert_eq!(ALL_KINDS.len(), 61, "palette registry must have 61 widgets");
    }

    #[test]
    fn each_kind_has_unique_kebab_type() {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for m in ALL_KINDS {
            assert!(
                seen.insert(m.kebab_type),
                "duplicate kebab_type: {}",
                m.kebab_type
            );
        }
        assert_eq!(seen.len(), 61);
    }

    #[test]
    fn factory_produces_kind_that_serializes_to_kebab_type() {
        use crate::types::config::{WidgetItem, WidgetStyleOverride};
        for m in ALL_KINDS {
            let cfg = (m.factory)();
            let item = WidgetItem {
                kind: cfg,
                style: WidgetStyleOverride::default(),
                raw_value: false,
            };
            let json = serde_json::to_value(&item).unwrap();
            let actual = json.get("type").and_then(|v| v.as_str()).unwrap();
            assert_eq!(
                actual, m.kebab_type,
                "factory for {} produced kebab type {}",
                m.display, actual
            );
        }
    }

    #[test]
    fn lookup_by_kebab_finds_known_kinds() {
        assert!(lookup_by_kebab("git-branch").is_some());
        assert!(lookup_by_kebab("session-cost").is_some());
        assert!(lookup_by_kebab("nonexistent").is_none());
    }

    #[test]
    fn each_category_has_at_least_one_entry() {
        use WidgetCategory::*;
        for cat in [
            Model, Static, Trivial, Session, Context, Worktree, Git, Transcript, Usage, Env,
        ] {
            assert!(
                ALL_KINDS.iter().any(|m| m.category == cat),
                "category {cat:?} has no entries"
            );
        }
    }
}
