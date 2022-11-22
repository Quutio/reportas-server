-- Add up migration script here
CREATE TYPE resolution_enum AS ENUM ('positive', 'neutral', 'negative');

CREATE TABLE users (
    id uuid primary key
);

CREATE TABLE reports (
    id bigserial primary key,
    active bool default 't' not null,
    timestamp timestamp not null,
    reporter_id uuid not null references users(id),
    reported_id uuid not null references users(id),
    handler_id uuid references users(id),
    handle_ts timestamp,
    comment text,
    description text not null,
    resolution resolution_enum
);

CREATE TABLE tags (
    id bigserial primary key,
    content text not null unique
);

CREATE TABLE tagmap (
    id bigserial primary key,
    report_id bigint references reports(id),
    tag_id bigint references tags(id)
);
