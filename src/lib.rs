pub mod cli;
pub mod commands;
pub mod hooks;
pub mod presentation;
pub mod recipe;
pub mod ui;
pub mod utils;

pub use utils::*;

rust_i18n::i18n!("locales", fallback = "en");
