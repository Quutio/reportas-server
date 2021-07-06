-- Your SQL goes here
CREATE TABLE reports (
    id BIGSERIAL PRIMARY KEY,
    active BOOL DEFAULT 't' NOT NULL,
    timestamp BIGINT NOT NULL,
    reporter TEXT NOT NULL,
    reported TEXT NOT NULL,
    handler TEXT,
    handle_ts BIGINT,
    description TEXT NOT NULL
)
