//! Core: process supervisor, app state, lifecycle.
//!
//! `AppState` is the single object installed into Tauri's managed state.
//! Every IPC command reads and mutates the world through it. Owning the
//! database pool, the engine, and the supervisor in one place keeps the
//! lifecycle legible.

use std::path::PathBuf;
use std::sync::Arc;

use sqlx::SqlitePool;

use crate::agents::engine::AgentEngine;
use crate::agents::supervisor::SharedSupervisor;

/// Everything IPC commands need to talk to the world.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub engine: Arc<dyn AgentEngine>,
    pub supervisor: SharedSupervisor,
    pub data_dir: PathBuf,
}
