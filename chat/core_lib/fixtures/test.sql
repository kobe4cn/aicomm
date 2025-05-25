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
