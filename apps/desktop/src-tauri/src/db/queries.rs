//! CRUD helpers for Phase 1. Query strings are kept short and typed at the
//! call site with `query_as`. Compile-time checked queries (`query!`) are a
//! future change once the schema stabilizes.

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use super::models::{Agent, Conversation, Message, MessageRole};
use super::DbError;

pub struct NewAgent<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub emoji: &'a str,
    pub color: &'a str,
    pub working_dir: &'a str,
    pub model_override: Option<&'a str>,
}

pub async fn insert_agent(pool: &SqlitePool, new: NewAgent<'_>) -> Result<Agent, DbError> {
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO agents (id, name, emoji, color, working_dir, model_override, status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, 'idle', ?, ?)",
    )
    .bind(new.id)
    .bind(new.name)
    .bind(new.emoji)
    .bind(new.color)
    .bind(new.working_dir)
    .bind(new.model_override)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    get_agent(pool, new.id)
        .await?
        .ok_or_else(|| DbError::Sqlx(sqlx::Error::RowNotFound))
}

pub async fn get_agent(pool: &SqlitePool, id: &str) -> Result<Option<Agent>, DbError> {
    let row = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn list_agents(pool: &SqlitePool) -> Result<Vec<Agent>, DbError> {
    let rows = sqlx::query_as::<_, Agent>("SELECT * FROM agents ORDER BY created_at ASC")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn update_agent_session_id(
    pool: &SqlitePool,
    id: &str,
    session_id: &str,
) -> Result<(), DbError> {
    sqlx::query("UPDATE agents SET session_id = ?, updated_at = ? WHERE id = ?")
        .bind(session_id)
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_agent_status(pool: &SqlitePool, id: &str, status: &str) -> Result<(), DbError> {
    sqlx::query("UPDATE agents SET status = ?, updated_at = ? WHERE id = ?")
        .bind(status)
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_agent(pool: &SqlitePool, id: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM agents WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn insert_conversation(
    pool: &SqlitePool,
    id: &str,
    agent_id: &str,
) -> Result<Conversation, DbError> {
    let now = Utc::now();
    sqlx::query("INSERT INTO conversations (id, agent_id, created_at) VALUES (?, ?, ?)")
        .bind(id)
        .bind(agent_id)
        .bind(now)
        .execute(pool)
        .await?;
    Ok(Conversation {
        id: id.to_string(),
        agent_id: agent_id.to_string(),
        created_at: now,
    })
}

pub async fn get_or_create_conversation_for_agent(
    pool: &SqlitePool,
    agent_id: &str,
) -> Result<Conversation, DbError> {
    if let Some(existing) = sqlx::query_as::<_, Conversation>(
        "SELECT * FROM conversations WHERE agent_id = ? ORDER BY created_at ASC LIMIT 1",
    )
    .bind(agent_id)
    .fetch_optional(pool)
    .await?
    {
        return Ok(existing);
    }

    let id = uuid::Uuid::new_v4().to_string();
    insert_conversation(pool, &id, agent_id).await
}

pub struct NewMessage<'a> {
    pub id: &'a str,
    pub conversation_id: &'a str,
    pub role: MessageRole,
    pub content: &'a str,
    pub created_at: DateTime<Utc>,
}

pub async fn insert_message(pool: &SqlitePool, new: NewMessage<'_>) -> Result<Message, DbError> {
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(new.id)
    .bind(new.conversation_id)
    .bind(new.role.as_str())
    .bind(new.content)
    .bind(new.created_at)
    .execute(pool)
    .await?;

    Ok(Message {
        id: new.id.to_string(),
        conversation_id: new.conversation_id.to_string(),
        role: new.role.as_str().to_string(),
        content: new.content.to_string(),
        created_at: new.created_at,
    })
}

pub async fn list_messages_for_agent(
    pool: &SqlitePool,
    agent_id: &str,
    limit: i64,
) -> Result<Vec<Message>, DbError> {
    let rows = sqlx::query_as::<_, Message>(
        "SELECT m.* FROM messages m
         JOIN conversations c ON c.id = m.conversation_id
         WHERE c.agent_id = ?
         ORDER BY m.created_at ASC
         LIMIT ?",
    )
    .bind(agent_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    async fn memory_pool() -> sqlx::SqlitePool {
        let opts = sqlx::sqlite::SqliteConnectOptions::new()
            .in_memory(true)
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn insert_and_get_agent_roundtrip() {
        let pool = memory_pool().await;
        let agent = insert_agent(
            &pool,
            NewAgent {
                id: "agent-1",
                name: "Scout",
                emoji: "🛰️",
                color: "#5E6AD2",
                working_dir: "/tmp/scout",
                model_override: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(agent.name, "Scout");
        assert_eq!(agent.status, "idle");
        assert!(agent.session_id.is_none());

        let fetched = get_agent(&pool, "agent-1").await.unwrap().unwrap();
        assert_eq!(fetched.id, agent.id);
        assert_eq!(fetched.created_at, agent.created_at);
    }

    #[tokio::test]
    async fn update_session_id_persists() {
        let pool = memory_pool().await;
        insert_agent(
            &pool,
            NewAgent {
                id: "a",
                name: "A",
                emoji: "🌟",
                color: "#5E6AD2",
                working_dir: "/tmp",
                model_override: None,
            },
        )
        .await
        .unwrap();

        update_agent_session_id(&pool, "a", "sess-123")
            .await
            .unwrap();
        let got = get_agent(&pool, "a").await.unwrap().unwrap();
        assert_eq!(got.session_id.as_deref(), Some("sess-123"));
    }

    #[tokio::test]
    async fn get_or_create_conversation_is_stable() {
        let pool = memory_pool().await;
        insert_agent(
            &pool,
            NewAgent {
                id: "a",
                name: "A",
                emoji: "🌟",
                color: "#5E6AD2",
                working_dir: "/tmp",
                model_override: None,
            },
        )
        .await
        .unwrap();

        let conv1 = get_or_create_conversation_for_agent(&pool, "a")
            .await
            .unwrap();
        let conv2 = get_or_create_conversation_for_agent(&pool, "a")
            .await
            .unwrap();
        assert_eq!(conv1.id, conv2.id);
    }

    #[tokio::test]
    async fn messages_list_in_order_for_agent() {
        let pool = memory_pool().await;
        insert_agent(
            &pool,
            NewAgent {
                id: "a",
                name: "A",
                emoji: "🌟",
                color: "#5E6AD2",
                working_dir: "/tmp",
                model_override: None,
            },
        )
        .await
        .unwrap();
        let conv = get_or_create_conversation_for_agent(&pool, "a")
            .await
            .unwrap();

        let t0 = Utc::now();
        for i in 0..3 {
            insert_message(
                &pool,
                NewMessage {
                    id: &format!("m-{i}"),
                    conversation_id: &conv.id,
                    role: MessageRole::User,
                    content: &format!("{{\"text\":\"hello {i}\"}}"),
                    created_at: t0 + chrono::Duration::milliseconds(i),
                },
            )
            .await
            .unwrap();
        }

        let msgs = list_messages_for_agent(&pool, "a", 200).await.unwrap();
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0].id, "m-0");
        assert_eq!(msgs[2].id, "m-2");
    }

    #[tokio::test]
    async fn delete_agent_cascades_to_conversations_and_messages() {
        let pool = memory_pool().await;
        insert_agent(
            &pool,
            NewAgent {
                id: "a",
                name: "A",
                emoji: "🌟",
                color: "#5E6AD2",
                working_dir: "/tmp",
                model_override: None,
            },
        )
        .await
        .unwrap();
        let conv = get_or_create_conversation_for_agent(&pool, "a")
            .await
            .unwrap();
        insert_message(
            &pool,
            NewMessage {
                id: "m-1",
                conversation_id: &conv.id,
                role: MessageRole::User,
                content: "{}",
                created_at: Utc::now(),
            },
        )
        .await
        .unwrap();

        delete_agent(&pool, "a").await.unwrap();

        let remaining: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(remaining.0, 0);
    }
}
