-- Add migration script here

--modify chat table to add agents column
ALTER TABLE chats ADD COLUMN agents BIGINT[] NOT NULL DEFAULT '{}';

--modify message table to add modified_content column
ALTER TABLE messages ADD COLUMN modified_content TEXT DEFAULT NULL;

--add agent_type type
CREATE TYPE agent_type AS ENUM ('proxy', 'reply', 'tap');

-- add chat_agent table
CREATE TABLE IF NOT EXISTS chat_agents (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL,
    name TEXT NOT NULL,
    type agent_type NOT NULL DEFAULT 'reply',
    prompt TEXT NOT NULL,
    args JSONB NOT NULL DEFAULT '{}',
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz DEFAULT CURRENT_TIMESTAMP
);
