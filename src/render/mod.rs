//! Renderer trait — joins widget segments into a single line.
//! Phase 2 ships only `Plain`. `Powerline` lands in Phase 4.

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub trait Renderer {
    fn render(&self, segments: &[String]) -> String;
}

pub struct Plain {
    pub separator: String,
}

impl Renderer for Plain {
    fn render(&self, segments: &[String]) -> String {
        segments
            .iter()
            .filter(|s| !s.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join(&self.separator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_with_separator() {
        let r = Plain {
            separator: " | ".into(),
        };
        let out = r.render(&["a".into(), "b".into(), "c".into()]);
        assert_eq!(out, "a | b | c");
    }

    #[test]
    fn filters_empty_segments() {
        let r = Plain {
            separator: " | ".into(),
        };
        let out = r.render(&["a".into(), String::new(), "b".into()]);
        assert_eq!(out, "a | b");
    }

    #[test]
    fn empty_input_yields_empty_string() {
        let r = Plain {
            separator: " | ".into(),
        };
        assert_eq!(r.render(&[]), "");
    }
}
