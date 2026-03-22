FROM rust:slim AS builder
WORKDIR /app/backend

RUN apt-get update && apt-get install -y --no-install-recommends curl && rm -rf /var/lib/apt/lists/*

COPY backend/Cargo.toml ./Cargo.toml
COPY backend/src ./src
COPY backend/migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/backend/target/release/backend /usr/local/bin/backend

EXPOSE 8080
CMD ["backend"]
