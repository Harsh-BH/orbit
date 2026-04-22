//! `#[tauri::command]` handlers for the frontend.
//!
//! Each command is a small adapter: validate input, call into the domain
//! modules (db, agents), format errors as user-facing strings, emit
//! side-channel events where needed.

use std::path::PathBuf;

use chrono::Utc;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::agents::engine::{AgentEvent, AgentId, EngineHealth, SpawnConfig};
use crate::agents::supervisor::SupervisedEvent;
use crate::core::AppState;
use crate::db::models::{Agent, Message, MessageRole};
use crate::db::queries::{self, NewAgent, NewMessage};

use super::events::{
    AgentEventPayload, AgentStatusChangePayload, AgentTerminatedPayload, EVENT_AGENT_EVENT,
    EVENT_AGENT_STATUS_CHANGE, EVENT_AGENT_TERMINATED,
};

/// User-facing command error type. Anything that reaches the frontend is
/// a human-readable string — the UI renders it verbatim.
pub type CommandResult<T> = Result<T, String>;

fn err<E: std::fmt::Display>(prefix: &str, e: E) -> String {
    format!("{prefix}: {e}")
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnAgentInput {
    pub name: String,
    pub emoji: String,
    pub color: String,
    pub working_dir: PathBuf,
    #[serde(default)]
    pub model_override: Option<String>,
    /// Canvas position at which to place the new agent. Defaults to the
    /// origin if omitted — clients that spawn from the canvas always
    /// pass the clicked point.
    #[serde(default)]
    pub position_x: f64,
    #[serde(default)]
    pub position_y: f64,
}

#[tauri::command]
pub async fn agent_spawn(
    state: State<'_, AppState>,
    app: AppHandle,
    input: SpawnAgentInput,
) -> CommandResult<Agent> {
    if input.name.trim().is_empty() {
        return Err("Agent name cannot be empty.".to_string());
    }
    if !input.working_dir.exists() {
        return Err(format!(
            "Working directory does not exist: {}",
            input.working_dir.display()
        ));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let working_dir_str = input.working_dir.to_string_lossy().to_string();

    // Phase 2: soft cap on concurrent agents. We revisit this in later
    // phases; the cap prevents a user from stumbling into OS-level
    // resource issues while the supervisor matures.
    const MAX_AGENTS: i64 = 10;
    let current = queries::count_agents(&state.pool)
        .await
        .map_err(|e| err("Failed to count agents", e))?;
    if current >= MAX_AGENTS {
        return Err(format!(
            "You already have {current} agents running. Terminate some before spawning more (limit: {MAX_AGENTS}).",
        ));
    }

    let agent = queries::insert_agent(
        &state.pool,
        NewAgent {
            id: &id,
            name: &input.name,
            emoji: &input.emoji,
            color: &input.color,
            working_dir: &working_dir_str,
            model_override: input.model_override.as_deref(),
            position_x: input.position_x,
            position_y: input.position_y,
        },
    )
    .await
    .map_err(|e| err("Failed to record agent", e))?;

    // Ensure a conversation exists so send_message doesn't have to worry
    // about creating one under a race.
    queries::get_or_create_conversation_for_agent(&state.pool, &id)
        .await
        .map_err(|e| err("Failed to initialize conversation", e))?;

    state
        .engine
        .spawn(SpawnConfig {
            agent_id: id.clone(),
            working_dir: input.working_dir,
            model_override: input.model_override,
            resume_session_id: None,
        })
        .await
        .map_err(|e| e.user_facing())?;

    queries::update_agent_status(&state.pool, &id, "idle")
        .await
        .map_err(|e| err("Failed to set status", e))?;

    let _ = app.emit(
        EVENT_AGENT_STATUS_CHANGE,
        AgentStatusChangePayload {
            agent_id: id.clone(),
            status: "idle".to_string(),
        },
    );

    Ok(agent)
}

#[tauri::command]
pub async fn agent_list(state: State<'_, AppState>) -> CommandResult<Vec<Agent>> {
    queries::list_agents(&state.pool)
        .await
        .map_err(|e| err("Failed to list agents", e))
}

#[tauri::command]
pub async fn agent_get_conversation(
    state: State<'_, AppState>,
    agent_id: AgentId,
) -> CommandResult<Vec<Message>> {
    queries::list_messages_for_agent(&state.pool, &agent_id, 200)
        .await
        .map_err(|e| err("Failed to load conversation", e))
}

#[tauri::command]
pub async fn agent_send_message(
    state: State<'_, AppState>,
    app: AppHandle,
    agent_id: AgentId,
    message: String,
) -> CommandResult<()> {
    if message.trim().is_empty() {
        return Err("Cannot send an empty message.".to_string());
    }
    let agent = queries::get_agent(&state.pool, &agent_id)
        .await
        .map_err(|e| err("Failed to look up agent", e))?
        .ok_or_else(|| format!("Agent {agent_id} not found."))?;

    let conversation = queries::get_or_create_conversation_for_agent(&state.pool, &agent.id)
        .await
        .map_err(|e| err("Failed to resolve conversation", e))?;

    // Persist the user message first (write-then-emit).
    let user_message_id = uuid::Uuid::new_v4().to_string();
    let user_content = serde_json::json!({ "text": message }).to_string();
    queries::insert_message(
        &state.pool,
        NewMessage {
            id: &user_message_id,
            conversation_id: &conversation.id,
            role: MessageRole::User,
            content: &user_content,
            created_at: Utc::now(),
        },
    )
    .await
    .map_err(|e| err("Failed to persist user message", e))?;

    // Broadcast a status change so the UI can show "active".
    queries::update_agent_status(&state.pool, &agent.id, "active")
        .await
        .ok();
    let _ = app.emit(
        EVENT_AGENT_STATUS_CHANGE,
        AgentStatusChangePayload {
            agent_id: agent.id.clone(),
            status: "active".to_string(),
        },
    );

    let stream = state
        .engine
        .send_message(&agent.id, &message)
        .await
        .map_err(|e| e.user_facing())?;

    let app_handle = app.clone();
    let pool = state.pool.clone();
    let supervisor_tx = state.supervisor.sender();
    let agent_id_for_task = agent.id.clone();
    let conversation_id_for_task = conversation.id.clone();

    tokio::spawn(async move {
        let mut stream = stream;
        let mut assistant_text = String::new();

        while let Some(event) = stream.next().await {
            let _ = app_handle.emit(
                EVENT_AGENT_EVENT,
                AgentEventPayload {
                    agent_id: agent_id_for_task.clone(),
                    event: event.clone(),
                },
            );
            let _ = supervisor_tx.send(SupervisedEvent {
                agent_id: agent_id_for_task.clone(),
                event: event.clone(),
            });

            match &event {
                AgentEvent::SessionStarted { session_id } => {
                    if let Err(e) =
                        queries::update_agent_session_id(&pool, &agent_id_for_task, session_id)
                            .await
                    {
                        tracing::warn!(error = %e, "failed to persist session_id");
                    }
                }
                AgentEvent::TextDelta { content } => {
                    assistant_text.push_str(content);
                }
                AgentEvent::ThinkingDelta { .. } => {
                    // Phase 3: persist thinking so it can be replayed.
                }
                AgentEvent::ToolUseComplete {
                    tool_id,
                    tool_name,
                    input,
                } => {
                    let content_json = serde_json::json!({
                        "tool_id": tool_id,
                        "tool_name": tool_name,
                        "input": input,
                    })
                    .to_string();
                    let id = uuid::Uuid::new_v4().to_string();
                    if let Err(e) = queries::insert_message(
                        &pool,
                        NewMessage {
                            id: &id,
                            conversation_id: &conversation_id_for_task,
                            role: MessageRole::ToolUse,
                            content: &content_json,
                            created_at: Utc::now(),
                        },
                    )
                    .await
                    {
                        tracing::warn!(error = %e, "failed to persist tool_use");
                    }
                }
                AgentEvent::ToolUseResult {
                    tool_id,
                    result,
                    is_error,
                } => {
                    let content_json = serde_json::json!({
                        "tool_id": tool_id,
                        "result": result,
                        "is_error": is_error,
                    })
                    .to_string();
                    let id = uuid::Uuid::new_v4().to_string();
                    if let Err(e) = queries::insert_message(
                        &pool,
                        NewMessage {
                            id: &id,
                            conversation_id: &conversation_id_for_task,
                            role: MessageRole::ToolResult,
                            content: &content_json,
                            created_at: Utc::now(),
                        },
                    )
                    .await
                    {
                        tracing::warn!(error = %e, "failed to persist tool_result");
                    }
                }
                AgentEvent::TurnComplete { .. } => {
                    if !assistant_text.is_empty() {
                        let content_json =
                            serde_json::json!({ "text": assistant_text }).to_string();
                        let id = uuid::Uuid::new_v4().to_string();
                        if let Err(e) = queries::insert_message(
                            &pool,
                            NewMessage {
                                id: &id,
                                conversation_id: &conversation_id_for_task,
                                role: MessageRole::Assistant,
                                content: &content_json,
                                created_at: Utc::now(),
                            },
                        )
                        .await
                        {
                            tracing::warn!(error = %e, "failed to persist assistant message");
                        }
                    }
                    let _ = queries::update_agent_status(&pool, &agent_id_for_task, "idle").await;
                    let _ = app_handle.emit(
                        EVENT_AGENT_STATUS_CHANGE,
                        AgentStatusChangePayload {
                            agent_id: agent_id_for_task.clone(),
                            status: "idle".to_string(),
                        },
                    );
                    break;
                }
                AgentEvent::Error { .. } => {
                    let _ = queries::update_agent_status(&pool, &agent_id_for_task, "error").await;
                    let _ = app_handle.emit(
                        EVENT_AGENT_STATUS_CHANGE,
                        AgentStatusChangePayload {
                            agent_id: agent_id_for_task.clone(),
                            status: "error".to_string(),
                        },
                    );
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn agent_terminate(
    state: State<'_, AppState>,
    app: AppHandle,
    agent_id: AgentId,
) -> CommandResult<()> {
    state
        .engine
        .terminate(&agent_id)
        .await
        .map_err(|e| e.user_facing())?;
    let _ = queries::update_agent_status(&state.pool, &agent_id, "idle").await;
    let _ = app.emit(
        EVENT_AGENT_TERMINATED,
        AgentTerminatedPayload {
            agent_id: agent_id.clone(),
            reason: "user_requested".to_string(),
        },
    );
    Ok(())
}

#[tauri::command]
pub async fn agent_delete(state: State<'_, AppState>, agent_id: AgentId) -> CommandResult<()> {
    // Best-effort termination — ignore errors (agent may not be running).
    let _ = state.engine.terminate(&agent_id).await;
    queries::delete_agent(&state.pool, &agent_id)
        .await
        .map_err(|e| err("Failed to delete agent", e))
}

#[tauri::command]
pub async fn agent_update_position(
    state: State<'_, AppState>,
    agent_id: AgentId,
    x: f64,
    y: f64,
) -> CommandResult<()> {
    queries::update_agent_position(&state.pool, &agent_id, x, y)
        .await
        .map_err(|e| err("Failed to update agent position", e))
}

#[tauri::command]
pub async fn agent_rename(
    state: State<'_, AppState>,
    agent_id: AgentId,
    name: String,
) -> CommandResult<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Agent name cannot be empty.".to_string());
    }
    queries::update_agent_name(&state.pool, &agent_id, trimmed)
        .await
        .map_err(|e| err("Failed to rename agent", e))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemHealth {
    pub engine: EngineHealth,
}

#[tauri::command]
pub async fn system_health_check(state: State<'_, AppState>) -> CommandResult<SystemHealth> {
    let engine = state
        .engine
        .health_check()
        .await
        .map_err(|e| e.user_facing())?;
    Ok(SystemHealth { engine })
}
