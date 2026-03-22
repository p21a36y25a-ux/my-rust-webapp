FROM rust:slim AS builder
WORKDIR /app/frontend

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk wasm-bindgen-cli

COPY frontend/Cargo.toml ./Cargo.toml
COPY frontend/Trunk.toml ./Trunk.toml
COPY frontend/index.html ./index.html
COPY frontend/src ./src

RUN trunk build --release

FROM nginx:alpine
COPY --from=builder /app/frontend/dist /usr/share/nginx/html

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
