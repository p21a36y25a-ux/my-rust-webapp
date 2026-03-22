# my-rust-webapp

[![CI](https://github.com/p21a36y25a-ux/my-rust-webapp/actions/workflows/ci.yml/badge.svg)](https://github.com/p21a36y25a-ux/my-rust-webapp/actions/workflows/ci.yml)

Concise full-stack starter with Axum backend, Yew WASM frontend, Postgres, Docker Compose, and GitHub Actions CI.

## Highlights

- Axum 0.7 REST backend
- Yew 0.21 CSR frontend
- SQLx + Postgres integration
- Docker Compose one-command local stack
- CI workflow for backend and frontend builds

## Repository Layout

- backend/ Axum + SQLx API
- frontend/ Yew CSR app built with Trunk
- .github/workflows/ci.yml CI build
- docker-compose.yml local multi-service stack

## Quick Start (Docker)

```bash
docker compose up --build
```

### Services

- Frontend: http://localhost:3000
- Backend: http://localhost:8080
- Postgres: localhost:5432

## Quick Start (Local)

1. Start Postgres (for example with Docker):

```bash
docker run --name mydb -e POSTGRES_PASSWORD=postgres -e POSTGRES_USER=postgres -e POSTGRES_DB=mydb -p 5432:5432 -d postgres:15
```

2. Run backend:

```bash
cd backend
set DATABASE_URL=postgres://postgres:postgres@localhost:5432/mydb
cargo run
```

3. Run frontend:

```bash
cd frontend
cargo install trunk
rustup target add wasm32-unknown-unknown
trunk serve --port 3000
```

Open http://localhost:3000 after both backend and frontend are running.

## API Endpoints

- `GET /api/health`
- `POST /api/users`
- `GET /api/users`

Create user example:

```bash
curl -X POST http://localhost:8080/api/users ^
  -H "Content-Type: application/json" ^
  -d "{\"name\":\"Ada\",\"email\":\"ada@example.com\"}"
```

List users:

```bash
curl http://localhost:8080/api/users
```

## Notes

- Backend runs SQL migrations from backend/migrations on startup.
- Frontend currently fetches users from http://localhost:8080/api/users.
- `sqlx-data.json` can be generated later if you switch to SQLx offline query macros.
