CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    email VARCHAR(254) NOT NULL UNIQUE CHECK (email = lower(email)),
    password TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'member' CHECK (role IN ('admin', 'member')),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive')),
    created_at TEXT NOT NULL DEFAULT to_char(CURRENT_TIMESTAMP AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
);
CREATE TABLE sessions (
    id VARCHAR(64) PRIMARY KEY,
    csrf VARCHAR(64) NOT NULL,
    user_id BIGINT REFERENCES users(id) ON DELETE CASCADE,
    expires_at BIGINT NOT NULL
);
CREATE INDEX sessions_expiry ON sessions(expires_at);
CREATE TABLE login_attempts (
    email VARCHAR(254) PRIMARY KEY,
    attempts BIGINT NOT NULL DEFAULT 0,
    started_at BIGINT NOT NULL
);
