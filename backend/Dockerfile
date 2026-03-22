FROM rust:1.70-slim AS builder
WORKDIR /app/backend

COPY backend/Cargo.toml ./Cargo.toml
COPY backend/src ./src
COPY backend/migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/backend/target/release/backend /usr/local/bin/backend

EXPOSE 8080
CMD ["backend"]
