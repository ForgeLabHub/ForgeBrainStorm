-- Per-user game results, one row per finished game and player.
-- Backs the "recent games" list on the profile page.
CREATE TABLE IF NOT EXISTS user_games (
    id            BIGSERIAL PRIMARY KEY,
    user_id       BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    room_code     TEXT NOT NULL,
    topic         TEXT NOT NULL,
    mode          TEXT NOT NULL,
    score         BIGINT NOT NULL,
    place         INTEGER NOT NULL,
    players       INTEGER NOT NULL,
    duration_secs INTEGER NOT NULL DEFAULT 0,
    played_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_user_games_user ON user_games (user_id, played_at DESC);
