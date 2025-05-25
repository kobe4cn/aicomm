-- Add migration script here
--create user table
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(64) NOT NULL,
    --hash the argon2 password
    password_hash VARCHAR(97) NOT NULL,
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);
-- workspace for users
CREATE TABLE IF NOT EXISTS workspaces (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(64) NOT NULL UNIQUE,
    owner_id BIGINT NOT NULL references users(id),
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);
--create index for users for email
CREATE UNIQUE INDEX IF NOT EXISTS email_index ON users(email);
--create chat type: single, group, private_channel, public_channel
create type chat_type as enum(
    'single',
    'group',
    'private_channel',
    'public_channel'
);
--create chat table
CREATE TABLE IF NOT EXISTS chats (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(64),
    type chat_type NOT NULL,
    --use id list
    members BIGINT [] NOT NULL,
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);
--create message table
CREATE TABLE IF NOT EXISTS messages (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL references chats(id),
    sender_id BIGINT NOT NULL references users(id),
    content TEXT NOT NULL,
    files TEXT [] default '{}',
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);
--create index for messages for chat_id and created_at by created_at desc
CREATE INDEX IF NOT EXISTS chat_id_created_at_index ON messages(chat_id, created_at DESC);
--create index for messages for sender_id
CREATE INDEX IF NOT EXISTS sender_id_index ON messages(sender_id, created_at DESC);
--alter users table for workspace
ALTER TABLE users
ADD COLUMN ws_id BIGINT references workspaces(id);
--alter chats table for workspace
ALTER TABLE chats
ADD COLUMN ws_id BIGINT references workspaces(id);
-- add super user 0 workspace 0
BEGIN;
INSERT INTO users (id, fullname, email, password_hash)
VALUES (0, 'super user', 'super@super.org', '');
INSERT INTO workspaces (id, name, owner_id)
VALUES (0, 'default', 0);
INSERT INTO chats(ws_id, name, type, members)
VALUES (0, 'default', 'public_channel', '{0}');
update users
set ws_id = 0
where id = 0;
alter table users
alter column ws_id
set not null;
COMMIT;
