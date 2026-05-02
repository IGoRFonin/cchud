//! Subcommands: cchud install (Task 7), cchud configure/import/doctor (later phases).

#[allow(dead_code)] // API для T9/T10 — пока не подключена к render-pipeline
pub mod env_loader; // Phase 7
pub mod install;
pub mod doctor; // Phase 9 Task 5

#[cfg(feature = "tui")]
pub mod configure;
#[cfg(feature = "tui")]
pub mod import;
