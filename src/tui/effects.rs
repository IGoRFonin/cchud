//! Reducer effects.

#![deny(clippy::unwrap_used, clippy::expect_used)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // RunInstall/Return* wired in T3/T6
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
    /// Home screen: Enter на "Install to Claude Code". Event loop вызывает
    /// `commands::install::install_idempotent` и сетит `app.status_message`.
    RunInstall,
    /// Save и возврат на Home (НЕ exit). Из `ConfirmReturnHome` modal.
    RequestSaveAndReturnHome,
    /// Discard editable и возврат на Home.
    RequestDiscardAndReturnHome,
}
