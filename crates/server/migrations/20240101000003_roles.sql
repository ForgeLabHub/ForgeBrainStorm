-- Roles: player, cheater, admin.
--
-- A role lives on the user, not on the session: one account, one role, and a
-- re-login never changes it. The old sessions.is_admin/is_tester pair allowed
-- two accounts to disagree about the same person and is dropped here.

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS role TEXT NOT NULL DEFAULT 'player';

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'users_role_check'
    ) THEN
        ALTER TABLE users
            ADD CONSTRAINT users_role_check
            CHECK (role IN ('player', 'cheater', 'admin'));
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_users_role ON users (role);

ALTER TABLE sessions DROP COLUMN IF EXISTS is_admin;
ALTER TABLE sessions DROP COLUMN IF EXISTS is_tester;
