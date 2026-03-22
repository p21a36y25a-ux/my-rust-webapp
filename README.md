# my-rust-webapp

[![CI](https://github.com/p21a36y25a-ux/my-rust-webapp/actions/workflows/ci.yml/badge.svg)](https://github.com/p21a36y25a-ux/my-rust-webapp/actions/workflows/ci.yml)

Production-oriented full-stack Time Attendance, HR and Payroll platform.

Backend: Rust (Axum + Tokio), SQLx/PostgreSQL, JWT auth, RBAC, tracing, OpenAPI/Swagger.
Frontend: Rust WASM (Yew) SPA with responsive dashboard, hover menus, branch home, attendance actions, vacation and payroll views.

## Core Features

- JWT auth with refresh flow and role-aware APIs.
- Roles: Employee, Manager, HR Admin, System Admin.
- Companies, branches, HR definitions, registrations, employees, leave requests, attendance, payroll runs.
- Attendance clock-in/out endpoint with camera photo reference storage.
- Real-time attendance feed endpoint (SSE) for dashboard updates.
- Payroll engine using Kosovo-style defaults:
  - 20 days/month * 8 hours/day = 160 standard hours
  - overtime threshold 160h
  - premium threshold 200h
  - EUR currency
- Leave workflow states: pending_manager -> pending_hr -> approved/denied.
- OpenAPI JSON and Swagger UI at /api/docs.
- Docker + docker-compose for local deployment.
- GitHub Actions CI build and test pipeline.

## Repository Layout

- backend/ Axum + SQLx API
- frontend/ Yew CSR app built with Trunk
- .github/workflows/ci.yml CI build
- docker-compose.yml local multi-service stack

## Menu Structure (UI)

- Employee:
  Register Employees, Click-in, Register Contracts, Employee Files, Employee Status
- Salary/Compensation:
  Salary Determination, Salary Period, Additional Days/Hours, Additional Income, Salary Calculation, Payroll List, E-Declaration (EDI)
- Vacation:
  Vacation Request, Vacation Hours, Holiday Status, Holiday Calendar
- Click-in/Click-out:
  Recording, Open entries/exits, list of clicks, Employees present
- HR Definitions:
  Employee status, contract types, employer type, vacation types, probation types, element calculation type, coefficient, salary elements
- Company:
  Company details, branches, departments/units, job positions
- Administration:
  municipal/state/bank registration, marital status

## Quick Start (Docker)

```bash
docker compose up --build
```

### Services

- Frontend: http://localhost:3000
- Backend: http://localhost:8080
- Postgres: localhost:5432
- Swagger: http://localhost:8080/api/docs

## Quick Start (Local)

1. Start Postgres:

```bash
docker run --name mydb -e POSTGRES_PASSWORD=postgres -e POSTGRES_USER=postgres -e POSTGRES_DB=mydb -p 5432:5432 -d postgres:15
```

2. Run backend:

```bash
cd backend
set DATABASE_URL=postgres://postgres:postgres@localhost:5432/mydb
set JWT_SECRET=super-secret-change-in-production
set UPLOAD_DIR=./uploads
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

## Demo Branches

- Prishtina
- Peja
- Prizreni

## Demo Accounts (Seeded)

All passwords are Argon2-hashed in the database. Default password for seeded users:

- Password: `Password123!`

Accounts:

- system_admin@example.com -> System Admin
- hr_admin@example.com -> HR Admin
- manager_prishtina@example.com -> Manager (Prishtina)
- employee_01_prishtina@example.com -> Employee

## API Endpoints

- `GET /api/health`
- `POST /api/auth/login`
- `POST /api/auth/refresh`
- `GET /api/company/branches`
- `GET /api/employees`
- `POST /api/employees`
- `POST /api/employees/{id}/files`
- `GET /api/attendance`
- `POST /api/attendance/punch`
- `GET /api/attendance/feed`
- `GET /api/leave`
- `POST /api/leave`
- `POST /api/leave/{id}/manager-decision`
- `POST /api/leave/{id}/hr-decision`
- `POST /api/payroll/calculate`
- `POST /api/payroll/run`
- `GET /api/payroll/{run_id}/edi`
- `GET /api/hr-definitions`
- `POST /api/hr-definitions`
- `GET /api/administration/registrations`

## Example Auth Login

```bash
curl -X POST http://localhost:8080/api/auth/login ^
  -H "Content-Type: application/json" ^
  -d "{\"email\":\"system_admin@example.com\",\"password\":\"Password123!\"}"
```

Take `access_token` and `csrf_token` from response, then call protected endpoints with:

- `Authorization: Bearer <access_token>`
- `x-csrf-token: <csrf_token>` for POST/PUT/PATCH/DELETE operations

## Payroll Rules Implemented

- Standard monthly hours: 160
- Overtime tier: hours above 160 up to 200, multiplier configurable (`tier2_rate_multiplier`)
- Premium tier: hours above 200, multiplier configurable (`tier3_rate_multiplier`)
- Formula includes bonus and deductions
- Returns computed totals + EDI line text

## Leave and Calendar

- Seeded holiday examples:
  Bajrami madh, Bajrami vogel, Krishtlindjet, Viti Ri, Dita e Pavarësis
- Workflow:
  Employee request -> Manager decision -> HR decision -> status finalized

## Security Notes

- Password hashing with Argon2
- JWT access + refresh tokens
- Role-based authorization checks
- CSRF header validation for state-changing requests
- SQLx parameterized queries
- File uploads stored in configured upload directory
- Audit records for payroll runs and sensitive corrections/actions

## Tooling

- Frontend dev: Trunk
- Backend dev: cargo-watch
- Migrations: sqlx migrate

Useful commands:

```bash
cd backend
cargo install sqlx-cli cargo-watch
sqlx migrate run
cargo watch -x run
```

## Testing

- Unit tests: payroll engine and role parsing in backend handlers
- Integration test scaffold in backend/tests/api_flows.rs (ignored by default)

Run tests:

```bash
cd backend
cargo test
```

## Notes

- Backend runs SQL migrations and seed data on startup.
- Frontend includes bilingual labels (Albanian + English) for key registration form fields.
- Attendance feed endpoint is designed for dashboard real-time presence updates.
- For production, set secure JWT secret and tighten CORS origins.
