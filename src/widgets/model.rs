//! Model widget — renders the model display name from the payload.

use crate::widgets::{RenderContext, Widget};

pub struct Model;

impl Widget for Model {
    fn id(&self) -> &'static str {
        "Model"
    }

    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let name = &ctx.payload.model.display_name;
        if name.is_empty() {
            None
        } else {
            Some(name.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::payload::{ModelInfo, StatusPayload, Workspace};

    fn payload_with_display_name(name: &str) -> StatusPayload {
        StatusPayload {
            session_id: "test".into(),
            model: ModelInfo {
                id: "claude-sonnet-4-6".into(),
                display_name: name.into(),
            },
            workspace: Workspace {
                current_dir: "/tmp".into(),
                project_dir: None,
                added_dirs: None,
            },
            transcript_path: None,
            cwd: None,
        }
    }

    #[test]
    fn renders_display_name() {
        let p = payload_with_display_name("Sonnet 4.6");
        let ctx = RenderContext::new(&p);
        assert_eq!(Model.render(&ctx), Some("Sonnet 4.6".into()));
    }

    #[test]
    fn returns_none_for_empty_name() {
        let p = payload_with_display_name("");
        let ctx = RenderContext::new(&p);
        assert_eq!(Model.render(&ctx), None);
    }
}
