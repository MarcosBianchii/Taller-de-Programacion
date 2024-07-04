pub mod commands;
pub mod config_file;
mod errors;
pub mod gitcommand;
pub mod logging;
mod objects;
pub mod plumbing;
pub mod protocol;
pub mod ui;

pub const DEFAULT_GIT_PORT: u32 = 9418; // Port generally used by Git, Source Daemon documentation
