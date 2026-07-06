    INSERT INTO users (id, first_name, last_name, email, password, status)
    VALUES (2, 'Test', 'User', 'test2@mairie360.fr', 'dummy', 'active')
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO users (id, first_name, last_name, email, password, status)
    VALUES (3, 'Test', 'User', 'test3@mairie360.fr', 'dummy', 'active')
    ON CONFLICT (id) DO NOTHING;
