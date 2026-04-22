//! Event names and payload types emitted from Rust to the frontend.
//!
//! Every event name is declared here so the TS side can import the
//! constants via a shared file or just string-match them. The payload
//! types are Serialize + Deserialize so they round-trip through Tauri's
//! serde bridge cleanly.

use serde::{Deserialize, Serialize};

use crate::agents::engine::{AgentEvent, AgentId};

pub const EVENT_AGENT_EVENT: &str = "agent:event";
pub const EVENT_AGENT_STATUS_CHANGE: &str = "agent:status_change";
pub const EVENT_AGENT_TERMINATED: &str = "agent:terminated";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEventPayload {
    pub agent_id: AgentId,
    pub event: AgentEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStatusChangePayload {
    pub agent_id: AgentId,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTerminatedPayload {
    pub agent_id: AgentId,
    pub reason: String,
}
