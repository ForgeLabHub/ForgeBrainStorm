-- BrainStorm initial schema.
-- Requires the `citext` extension (shipped in postgresql-contrib).
CREATE EXTENSION IF NOT EXISTS citext;

-- Designed to close the critical vulnerabilities found in the Python audit:
--   * bans keyed by (identifier, kind) cover IP, user_id, session hash
--   * audit_log is append-only for admin actions
--   * sessions use HMAC tokens, not plaintext cookies
--   * scores are tied to authenticated user_id, never display-name

-- ── Users ───────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS users (
    id           BIGSERIAL PRIMARY KEY,
    username     CITEXT NOT NULL UNIQUE,
    pwd_hash     TEXT NOT NULL,                  -- argon2id
    email        CITEXT,                        -- optional, for recovery
    coins        INTEGER NOT NULL DEFAULT 50,
    xp           BIGINT NOT NULL DEFAULT 0,
    games_played INTEGER NOT NULL DEFAULT 0,
    wins         INTEGER NOT NULL DEFAULT 0,
    total_score  BIGINT NOT NULL DEFAULT 0,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_username ON users (username);
CREATE INDEX IF NOT EXISTS idx_users_xp ON users (xp DESC);

-- ── Sessions ────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS sessions (
    token        TEXT PRIMARY KEY,              -- opaque random token
    user_id      BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_admin     BOOLEAN NOT NULL DEFAULT FALSE,
    is_tester    BOOLEAN NOT NULL DEFAULT FALSE,
    ip_address   INET,
    user_agent   TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at   TIMESTAMPTZ NOT NULL,
    revoked_at   TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions (expires_at);

-- ── Bans (IP, user, session-hash) ───────────────────────────
CREATE TABLE IF NOT EXISTS bans (
    id           BIGSERIAL PRIMARY KEY,
    identifier   TEXT NOT NULL,                 -- ip / user_id / session hash
    kind         TEXT NOT NULL CHECK (kind IN ('ip', 'user', 'sid_hash')),
    reason       TEXT NOT NULL DEFAULT '',
    issued_by    BIGINT REFERENCES users(id),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at   TIMESTAMPTZ NOT NULL,
    UNIQUE (identifier, kind)
);

CREATE INDEX IF NOT EXISTS idx_bans_active ON bans (identifier, kind, expires_at);

-- ── Audit log (append-only) ─────────────────────────────────
CREATE TABLE IF NOT EXISTS audit_log (
    id           BIGSERIAL PRIMARY KEY,
    actor_id     BIGINT REFERENCES users(id),
    actor_name   TEXT,                          -- denormalised in case user is deleted
    action       TEXT NOT NULL,                 -- ban, unban, impersonate, delete_user, ...
    target       TEXT NOT NULL,                 -- free-form: username, ip, room code
    reason       TEXT NOT NULL DEFAULT '',
    metadata     JSONB NOT NULL DEFAULT '{}',
    ip_address   INET,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_log (action);
CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_log (created_at DESC);

-- ── Leaderboard (materialised aggregates) ───────────────────
CREATE TABLE IF NOT EXISTS leaderboard (
    user_id      BIGINT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    username     CITEXT NOT NULL,
    total_score  BIGINT NOT NULL DEFAULT 0,
    games_played INTEGER NOT NULL DEFAULT 0,
    wins         INTEGER NOT NULL DEFAULT 0,
    total_time   INTEGER NOT NULL DEFAULT 0,
    last_seen    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_lb_score ON leaderboard (total_score DESC);

-- View: rank using window function (avoids correlated subqueries).
CREATE OR REPLACE VIEW leaderboard_ranked AS
SELECT
    user_id,
    username,
    total_score,
    games_played,
    wins,
    total_time,
    last_seen,
    RANK() OVER (ORDER BY total_score DESC) AS rank
FROM leaderboard;

-- ── Achievements ────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS achievements (
    id           BIGSERIAL PRIMARY KEY,
    user_id      BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key          TEXT NOT NULL,
    unlocked_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, key)
);

CREATE INDEX IF NOT EXISTS idx_achievements_user ON achievements (user_id);

-- ── Campaign progress ───────────────────────────────────────
CREATE TABLE IF NOT EXISTS campaign_progress (
    user_id      BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    campaign_id  TEXT NOT NULL,
    stage        INTEGER NOT NULL DEFAULT 0,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, campaign_id)
);

-- ── Room history (post-game snapshot) ───────────────────────
CREATE TABLE IF NOT EXISTS room_history (
    id           BIGSERIAL PRIMARY KEY,
    room_code    TEXT NOT NULL,
    mode         TEXT NOT NULL,
    topic        TEXT NOT NULL,
    duration_secs INTEGER NOT NULL,
    players_json JSONB NOT NULL DEFAULT '[]',
    questions_json JSONB NOT NULL DEFAULT '[]',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_room_history_code ON room_history (room_code);
CREATE INDEX IF NOT EXISTS idx_room_history_created ON room_history (created_at DESC);

-- ── UGC questions ───────────────────────────────────────────
CREATE TABLE IF NOT EXISTS ugc_questions (
    id           BIGSERIAL PRIMARY KEY,
    author_id    BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    topic        TEXT NOT NULL,
    difficulty   TEXT NOT NULL CHECK (difficulty IN ('easy', 'medium', 'hard')),
    question     TEXT NOT NULL,
    options      JSONB NOT NULL,                -- array of strings
    correct      INTEGER NOT NULL,
    explanation  TEXT NOT NULL DEFAULT '',
    hint         TEXT NOT NULL DEFAULT '',
    status       TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected')),
    reviewer_id  BIGINT REFERENCES users(id),
    review_note  TEXT NOT NULL DEFAULT '',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reviewed_at  TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_ugc_status ON ugc_questions (status);

-- ── Question cache (AI output reuse) ────────────────────────
CREATE TABLE IF NOT EXISTS question_cache (
    cache_key    TEXT PRIMARY KEY,              -- hash(topic|difficulty|num_options|mode)
    questions    JSONB NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at   TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_qcache_expires ON question_cache (expires_at);

-- ── Helpers: periodic cleanup ───────────────────────────────
-- Called by application or pg_cron; deletes expired sessions, bans, cache.
CREATE OR REPLACE FUNCTION cleanup_expired() RETURNS INTEGER AS $$
DECLARE
    removed INTEGER := 0;
    r1 INTEGER; r2 INTEGER; r3 INTEGER;
BEGIN
    DELETE FROM sessions WHERE expires_at < NOW() OR revoked_at IS NOT NULL;
    GET DIAGNOSTICS r1 = ROW_COUNT;
    DELETE FROM bans WHERE expires_at < NOW();
    GET DIAGNOSTICS r2 = ROW_COUNT;
    DELETE FROM question_cache WHERE expires_at < NOW();
    GET DIAGNOSTICS r3 = ROW_COUNT;
    removed := r1 + r2 + r3;
    RETURN removed;
END;
$$ LANGUAGE plpgsql;
