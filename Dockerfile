FROM rust:1.96.0-bookworm AS builder

WORKDIR /app
ENV SQLX_OFFLINE=true

COPY migrations ./migrations
COPY .sqlx ./.sqlx
COPY src ./src
COPY Cargo.lock ./Cargo.lock
COPY Cargo.toml ./Cargo.toml

RUN cargo build --release


FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/url-shortener ./url-shortener
CMD ["./url-shortener"]

