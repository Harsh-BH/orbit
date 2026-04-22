//! Database row types, serializable to the UI via serde.
//!
//! The same struct doubles as both the `FromRow` target and the wire
//! representation for Tauri commands. Fields reserved for later phases
//! (soul/purpose/memory, folder_access, team_id, position) are present but
//! not populated in Phase 1.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub type AgentId = String;
pub type ConversationId = String;
pub type MessageId = String;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: AgentId,
    pub name: String,
    pub emoji: String,
    pub color: String,
    pub working_dir: String,
    pub session_id: Option<String>,
    pub model_override: Option<String>,
    pub status: String,

    // Phase 3 — not read or written in Phase 1.
    pub soul: Option<String>,
    pub purpose: Option<String>,
    pub memory: Option<String>,

    // Phase 5 — stored as a JSON string in SQLite; empty array by default.
    pub folder_access: String,
    pub team_id: Option<String>,

    // Phase 2 — canvas position.
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: ConversationId,
    pub agent_id: AgentId,
    pub created_at: DateTime<Utc>,
}

/// Role string stored in the messages table. We keep it as a plain string
/// rather than an enum on the DB side so schema changes don't require a
/// migration when we add new kinds (e.g. `thinking` in Phase 3).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    ToolUse,
    ToolResult,
}

impl MessageRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
            Self::ToolUse => "tool_use",
            Self::ToolResult => "tool_result",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "user" => Some(Self::User),
            "assistant" => Some(Self::Assistant),
            "system" => Some(Self::System),
            "tool_use" => Some(Self::ToolUse),
            "tool_result" => Some(Self::ToolResult),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: MessageId,
    pub conversation_id: ConversationId,
    /// Stored as text, kept as a string on the Rust side so unknown future
    /// role variants don't cause hard decode errors on read.
    pub role: String,
    /// JSON-encoded payload. Shape depends on role:
    /// - `user` / `assistant` / `system`: `{ "text": "..." }`
    /// - `tool_use`: `{ "tool_id": "...", "tool_name": "...", "input": {...} }`
    /// - `tool_result`: `{ "tool_id": "...", "result": "...", "is_error": bool }`
    pub content: String,
    pub created_at: DateTime<Utc>,
}
