use crate::types::payload::{ModelInfo, StatusPayload, Workspace};

#[must_use]
pub fn payload_no_transcript() -> StatusPayload {
    StatusPayload {
        session_id: "test".into(),
        model: ModelInfo {
            id: "m".into(),
            display_name: "M".into(),
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
