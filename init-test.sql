-- Test fixtures run by the `seeder` service once Liquibase is done (dev, integration,
-- performance and security stacks).
--
-- User 1 (Admin) is already created by the `create_admin` changeset of the liquibase-migrations
-- image; it is the `sub` of the JWT ZAP injects. Users 2 and 3 are plain `User` accounts (chat
-- members).
--
-- The password is the public argon2id hash of the template admin account: `users.password` only
-- accepts argon2id PHC strings since database 1.3.0 (chk_users_password_hashed).

INSERT INTO users (id, first_name, last_name, email, password, status, is_archived)
VALUES
    (2, 'Test', 'User', 'test2@mairie360.fr',
     '$argon2id$v=19$m=19456,t=2,p=1$/iKF9PbiDRDs4EKPjlIIhg$UKx9vfwwps250mEP/bYp63CXbEnQGULeUAhDq+az9Aw',
     'active', FALSE),
    (3, 'Test', 'User', 'test3@mairie360.fr',
     '$argon2id$v=19$m=19456,t=2,p=1$/iKF9PbiDRDs4EKPjlIIhg$UKx9vfwwps250mEP/bYp63CXbEnQGULeUAhDq+az9Aw',
     'active', FALSE),
    (42, 'Jean', 'Dupont', 'jean.dupont@mairie360.fr',
     '$argon2id$v=19$m=19456,t=2,p=1$/iKF9PbiDRDs4EKPjlIIhg$UKx9vfwwps250mEP/bYp63CXbEnQGULeUAhDq+az9Aw',
     'active', FALSE)
ON CONFLICT (id) DO NOTHING;

INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id FROM roles r CROSS JOIN (VALUES (2), (3), (42)) AS u(id) WHERE r.name = 'User'
ON CONFLICT DO NOTHING;

-- Chat 5 and its message 118 are the ids of the path parameter examples of the spec (user 42 is a
-- member): ZAP builds its requests from these examples, so seeding them makes it scan the
-- handlers on a real chat instead of stopping at a 404.
INSERT INTO conversations (id, title, kind) VALUES (5, 'Service urbanisme', 'direct')
ON CONFLICT (id) DO NOTHING;

INSERT INTO conversation_members (conversation_id, user_id) VALUES (5, 1), (5, 42)
ON CONFLICT DO NOTHING;

INSERT INTO messages (id, conversation_id, owner_id, content)
VALUES (118, 5, 42, 'La réunion est décalée à 15h.')
ON CONFLICT (id) DO NOTHING;

-- Explicit ids do not advance the SERIAL sequence: move it past the fixtures so users created
-- later (Core_API, other fixtures) do not collide with them.
SELECT setval(pg_get_serial_sequence('users', 'id'), GREATEST((SELECT MAX(id) FROM users), 1));
SELECT setval(pg_get_serial_sequence('conversations', 'id'), GREATEST((SELECT MAX(id) FROM conversations), 1));
SELECT setval(pg_get_serial_sequence('messages', 'id'), GREATEST((SELECT MAX(id) FROM messages), 1));
