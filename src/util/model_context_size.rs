//! Model id → max context tokens lookup.
//!
//! Prefix-match — гибче статической мапы, новые модели ловятся одной
//! новой arm. Все Anthropic modern-models = 200k tokens; OSS-модели и
//! не-Anthropic id'ы возвращают None (виджет `ContextPercentageUsable`
//! gracefully отключается).

#![deny(clippy::unwrap_used, clippy::expect_used)]

#[must_use]
pub fn max_tokens_for(model_id: &str) -> Option<u64> {
    match model_id {
        s if s.starts_with("claude-opus-4") => Some(200_000),
        s if s.starts_with("claude-sonnet-4") => Some(200_000),
        s if s.starts_with("claude-haiku-4") => Some(200_000),
        s if s.starts_with("claude-3-5") => Some(200_000),
        s if s.starts_with("claude-3-opus") => Some(200_000),
        s if s.starts_with("claude-3-haiku") => Some(200_000),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn known_opus_4_family() {
        assert_eq!(max_tokens_for("claude-opus-4-7"), Some(200_000));
        assert_eq!(max_tokens_for("claude-opus-4-7[1m]"), Some(200_000));
    }

    #[test]
    fn known_sonnet_4_family() {
        assert_eq!(max_tokens_for("claude-sonnet-4-6"), Some(200_000));
    }

    #[test]
    fn known_haiku_4_family() {
        assert_eq!(max_tokens_for("claude-haiku-4-5-20251001"), Some(200_000));
    }

    #[test]
    fn known_3_5_family() {
        assert_eq!(max_tokens_for("claude-3-5-sonnet-20240620"), Some(200_000));
    }

    #[test]
    fn known_3_opus_family() {
        assert_eq!(max_tokens_for("claude-3-opus-20240229"), Some(200_000));
    }

    #[test]
    fn unknown_model_returns_none() {
        assert_eq!(max_tokens_for("gpt-4o"), None);
        assert_eq!(max_tokens_for("llama-3-70b"), None);
        assert_eq!(max_tokens_for(""), None);
    }
}
