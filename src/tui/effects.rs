//! Reducer effects.

#![deny(clippy::unwrap_used, clippy::expect_used)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReducerEffect {
    /// Никаких side effects, продолжаем event loop.
    None,
    /// User просит выход без save (нет dirty или подтвердил discard).
    Quit,
    /// User в `ConfirmQuit` нажал `s` — event loop делает save + exit.
    RequestSaveAndQuit,
    /// User в `ConfirmQuit` нажал `d` — event loop discard'ит и exit'ит.
    RequestDiscardAndQuit,
    /// Hint event loop'у что preview надо перерисовать. Сейчас preview lazy
    /// (на каждом draw frame) — оставлено как hint для future оптимизаций.
    RebuildPreview,
}
