-- Your SQL goes here
CREATE TABLE reports (
    id bigserial primary key,
    active bool default 't' not null,
    timestamp bigint not null,
    reporter text not null,
    reported text not null,
    handler text,
    handle_ts bigint,
    comment text,
    description text not null,
    tags text
)

CREATE TABLE tags (
    id bigserial primary key,
    content text
)

CREATE TABLE tagmap (
    id bigserial primary key,
    report_id bigint references reports(id),
    tag_id bigint references tags(id)
)
