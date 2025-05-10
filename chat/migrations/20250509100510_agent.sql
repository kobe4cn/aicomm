-- Add AdapterType enum
CREATE TYPE adapter_type AS ENUM ('openai', 'ollama');
-- Add adapter and model to chat_agents table
ALTER TABLE chat_agents
ADD COLUMN adapter adapter_type NOT NULL DEFAULT 'ollama',
    ADD COLUMN model VARCHAR(255) NOT NULL DEFAULT 'llama3.2';
