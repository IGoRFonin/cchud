//! Subcommands.

pub mod doctor;
pub mod env_loader;
pub mod install;

#[cfg(feature = "tui")]
pub mod configure;
#[cfg(feature = "tui")]
pub mod import;
