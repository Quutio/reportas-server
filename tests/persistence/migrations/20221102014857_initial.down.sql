-- Add down migration script here

DROP TYPE if EXISTS resolution_enum;

DROP TABLE if EXISTS users cascade;
DROP TABLE if EXISTS tags cascade;
DROP TABLE if EXISTS tagmap cascade;
DROP TABLE if EXISTS reports cascade;
