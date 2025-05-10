-- Add migration script here
--modify message table to change modified_content column support default value
ALTER TABLE messages ALTER COLUMN modified_content DROP DEFAULT;
