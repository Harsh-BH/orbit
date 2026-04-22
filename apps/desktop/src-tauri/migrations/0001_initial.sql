-- Initial schema for Orbit.
--
-- Idempotent (CREATE TABLE IF NOT EXISTS) so this migration is safe to
-- re-run against an already-initialized database. Future schema changes
-- land in a new numbered migration file — never edit this one.

CREATE TABLE IF NOT EXISTS agents (
    id             TEXT PRIMARY KEY NOT NULL,
    name           TEXT NOT NULL,
    emoji          TEXT NOT NULL,
    color          TEXT NOT NULL,
    working_dir    TEXT NOT NULL,
    session_id     TEXT,
    model_override TEXT,
    status         TEXT NOT NULL DEFAULT 'idle',

    -- Reserved for Phase 3 (agent identity).
    soul           TEXT,
    purpose        TEXT,
    memory         TEXT,

    -- Reserved for Phase 5 (teams / folder access).
    folder_access  TEXT NOT NULL DEFAULT '[]',
    team_id        TEXT,

    -- Reserved for Phase 2 (canvas position).
    position_x     REAL,
    position_y     REAL,

    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS conversations (
    id         TEXT PRIMARY KEY NOT NULL,
    agent_id   TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS messages (
    id              TEXT PRIMARY KEY NOT NULL,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role            TEXT NOT NULL,
    -- content is a JSON blob so we can store structured data for tool_use
    -- and tool_result rows alongside plain strings for user/assistant text.
    content         TEXT NOT NULL,
    created_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_conversations_agent_id
    ON conversations (agent_id);

CREATE INDEX IF NOT EXISTS idx_messages_conversation
    ON messages (conversation_id, created_at);
