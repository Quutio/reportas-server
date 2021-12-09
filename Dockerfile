FROM rust:1 as builder

RUN USER=root cargo new --bin reportas-server
RUN rustup component add rustfmt
WORKDIR ./reportas-server

RUN echo "fn main() {}" > src/lib.rs && echo "fn main() {}" > src/client.rs && echo "fn main() {}" > src/server.rs

COPY ./reportas-server/Cargo.lock ./Cargo.lock
COPY ./reportas-server/Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

ADD ./reportas-server ./

RUN rm ./target/release/deps/server*
RUN cargo build --release

FROM debian:buster-slim

ARG APP=/usr/src/app
ARG LISTEN_PORT=50051
ARG LISTEN_ADDR="[::1]"

ENV LISTEN_PORT=${LISTEN_PORT}
ENV LISTEN_ADDR=${LISTEN_ADDR}

ENV DATABASE_ADDR="localhost"
ENV DATABASE_PORT=5432
ENV DATABASE_URL="postgresql://reportas:passwd@localhost:5432/reportas"
ENV RUST_LOG="INFO"

EXPOSE ${LISTEN_PORT}

ENV APP_USER=appuser

RUN groupadd $APP_USER \
  && useradd -g $APP_USER $APP_USER \
  && mkdir -p ${APP}

RUN apt-get update && apt-get -y install libpq-dev && apt-get -y install postgresql

COPY --from=builder /reportas-server/target/release/server ${APP}/server
COPY --from=builder /reportas-server/migrations/latest_reports/up.sql ${APP}/up.sql

USER $APP_USER
WORKDIR ${APP}

CMD sleep 5 && psql ${DATABASE_URL} -f ./up.sql && ./server --address ${LISTEN_ADDR} --port ${LISTEN_PORT}
