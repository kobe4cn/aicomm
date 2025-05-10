-- insert workspaces
insert into workspaces (name, owner_id)
values ('workspace1', 0),
    ('workspace2', 0),
    ('workspace3', 0);
-- insert users
-- password: test123456
insert into users (fullname, email, password_hash, ws_id)
values (
        'kevin',
        'kevin.yang.xgz@gmail.com',
        '$argon2id$v=19$m=19456,t=2,p=1$uA3da3UQnoSVOFSwF4Aw3Q$9BF+ZDpP+cvERAjYnESkRFQ5GJU5OCb+0GQe3twXzqg',
        1
    ),
    (
        'kevin2',
        'kevin2.yang.xgz@gmail.com',
        '$argon2id$v=19$m=19456,t=2,p=1$uA3da3UQnoSVOFSwF4Aw3Q$9BF+ZDpP+cvERAjYnESkRFQ5GJU5OCb+0GQe3twXzqg',
        1
    ),
    (
        'kevin3',
        'kevin3.yang.xgz@gmail.com',
        '$argon2id$v=19$m=19456,t=2,p=1$uA3da3UQnoSVOFSwF4Aw3Q$9BF+ZDpP+cvERAjYnESkRFQ5GJU5OCb+0GQe3twXzqg',
        1
    );
-- insert chats
INSERT INTO chats(ws_id, name, type, members)
VALUES(1, 'general', 'public_channel', '{ 1, 2, 3 }'),
    (1, 'private', 'private_channel', '{ 1, 2 }');
-- insert chats no name
INSERT INTO chats(ws_id, type, members)
VALUES(1, 'single', '{ 1, 2 }'),
    (1, 'group', '{ 1, 2, 3 }');

--insert agent to chat
INSERT INTO chat_agents(chat_id, name, type, prompt, args )
VALUES(1, 'translation', 'proxy', 'If language is Chinese, translate it to English, if language is English, translate it to Chinese.Please reply the translated content directly, do not add any other text. Here is the content: ', '{}');
INSERT INTO chat_agents(chat_id, name, type, prompt, args )
VALUES(2, 'translation1', 'proxy', 'If language is Chinese, translate it to English, if language is English, translate it to Chinese.Please reply the translated content directly, do not add any other text. Here is the content: ', '{}');
INSERT INTO chat_agents(chat_id, name, type, prompt, args )
VALUES(2, 'translation2', 'proxy', 'You''re the world''s best translator,You understand English and Chinese well, also their culture and history. If the original text is English, you will translate it to elegant, authentic Simplified Chinese. If the original text is Chinese, you will translate it to elegant, authentic English. Only return the translated sentences, no other text or comments. Belows are the text to translate: ', '{}');
