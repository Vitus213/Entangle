-- Create super admin user
-- Default credentials: admin@entangle.local / admin123456
-- Please change the password after first login!
-- Password hash generated using Argon2id (matches entangle-auth implementation)

DO $$
DECLARE
    admin_role_id UUID;
    super_admin_id UUID;
    table_exists BOOLEAN;
BEGIN
    -- Check if roles table exists
    SELECT EXISTS (
        SELECT 1 FROM information_schema.tables
        WHERE table_name = 'roles'
    ) INTO table_exists;

    IF NOT table_exists THEN
        -- Create roles table
        CREATE TABLE roles (
            id UUID PRIMARY KEY,
            name VARCHAR(50) UNIQUE NOT NULL,
            description TEXT,
            is_system BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        -- Insert default roles
        INSERT INTO roles (id, name, description, is_system) VALUES
            ('00000000-0000-0000-0000-000000000001'::UUID, 'admin', '系统管理员，拥有所有权限', TRUE),
            ('00000000-0000-0000-0000-000000000002'::UUID, 'editor', '编辑者，可以创建和编辑文档', TRUE),
            ('00000000-0000-0000-0000-000000000003'::UUID, 'viewer', '查看者，只能查看文档', TRUE);

        -- Get the admin role ID we just inserted
        admin_role_id := '00000000-0000-0000-0000-000000000001'::UUID;
    ELSE
        -- Try to get admin role ID, use EXCEPTION block to handle missing
        BEGIN
            SELECT id INTO admin_role_id FROM roles WHERE name = 'admin' LIMIT 1;
        EXCEPTION WHEN NO_DATA_FOUND THEN
            -- Admin role doesn't exist, create it
            INSERT INTO roles (id, name, description, is_system)
            VALUES ('00000000-0000-0000-0000-000000000001'::UUID, 'admin', '系统管理员，拥有所有权限', TRUE);
            admin_role_id := '00000000-0000-0000-0000-000000000001'::UUID;
        END;
    END IF;

    -- Check if super admin already exists
    BEGIN
        SELECT id INTO super_admin_id FROM users WHERE email = 'admin@entangle.local' LIMIT 1;
    EXCEPTION WHEN NO_DATA_FOUND THEN
        super_admin_id := NULL;
    END;

    IF super_admin_id IS NULL THEN
        -- Create super admin user
        -- Password hash for 'admin123456' (Argon2id)
        INSERT INTO users (id, email, password_hash, nickname, role_id, email_verified, status)
        VALUES (
            '00000000-0000-0000-0000-000000000999'::UUID,
            'admin@entangle.local',
            '$argon2id$v=19$m=19456,t=2,p=1$Hzw1/64Dr+d5k338RwWJYg$fyrVWXaM0JJRLNmUlgf7RgumC8QUrooDI1kul4xRDv8',
            '超级管理员',
            admin_role_id,
            TRUE,
            'active'
        );

        RAISE NOTICE 'Super admin user created successfully.';
        RAISE NOTICE 'Email: admin@entangle.local';
        RAISE NOTICE 'Password: admin123456';
        RAISE NOTICE 'Please change the password after first login!';
    ELSE
        -- Update existing user to ensure they have admin role
        UPDATE users
        SET role_id = admin_role_id,
            status = 'active',
            nickname = '超级管理员',
            email_verified = TRUE
        WHERE id = super_admin_id;

        RAISE NOTICE 'Super admin user already exists. Updated to ensure admin role.';
    END IF;
END $$;
