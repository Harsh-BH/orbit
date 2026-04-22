//! Tauri command handlers exposed to the frontend.
//!
//! Commands are the only way the UI interacts with core state. Everything
//! is async, returns structured string errors (Tauri serializes), and is
//! registered in `lib.rs::run`.

pub mod commands;
pub mod events;

pub use commands::*;
pub use events::*;
