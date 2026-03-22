use std::net::SocketAddr;

use axum::{
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod auth;
mod db;
mod handlers;
mod models;
mod seed;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_keys: auth::JwtKeys,
    pub attendance_tx: tokio::sync::broadcast::Sender<models::AttendanceEvent>,
    pub upload_dir: String,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health,
        handlers::login,
        handlers::refresh,
        handlers::list_branches,
        handlers::list_employees,
        handlers::create_employee,
        handlers::upload_employee_file,
        handlers::attendance_punch,
        handlers::list_attendance,
        handlers::attendance_feed,
        handlers::leave_request,
        handlers::list_leave_requests,
        handlers::manager_decide_leave,
        handlers::hr_decide_leave,
        handlers::calculate_payroll,
        handlers::run_payroll,
        handlers::export_edi,
        handlers::list_hr_definitions,
        handlers::create_hr_definition,
        handlers::list_registrations,
    ),
    components(schemas(
        models::LoginRequest,
        models::RefreshRequest,
        models::AuthResponse,
        models::ApiMessage,
        models::Branch,
        models::Employee,
        models::EmployeeCreate,
        models::AttendanceRecord,
        models::AttendancePunchRequest,
        models::AttendanceEvent,
        models::LeaveRequestRecord,
        models::LeaveCreateRequest,
        models::LeaveDecisionRequest,
        models::PayrollInput,
        models::PayrollResult,
        handlers::HrDefinitionCreate,
        handlers::HrDefinitionRecord,
        handlers::RegistrationRecord,
        handlers::EmployeeQuery,
        handlers::AttendanceQuery,
        handlers::LeaveQuery,
    )),
    tags(
        (name = "time-attendance-hr-payroll", description = "Time attendance, HR and payroll APIs")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_owned());
    let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_owned());

    let pool = PgPool::connect(&database_url).await?;
    db::migrate(&pool).await?;
    tokio::fs::create_dir_all(&upload_dir).await?;

    let (attendance_tx, _attendance_rx) = tokio::sync::broadcast::channel(2048);
    let state = AppState {
        pool: pool.clone(),
        jwt_keys: auth::JwtKeys::from_secret(&jwt_secret),
        attendance_tx,
        upload_dir,
    };

    seed::seed_demo_data(&state).await?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    let api = Router::new()
        .route("/api/health", get(handlers::health))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/refresh", post(handlers::refresh))
        .route("/api/company/branches", get(handlers::list_branches))
        .route("/api/employees", get(handlers::list_employees).post(handlers::create_employee))
        .route("/api/employees/:id/files", post(handlers::upload_employee_file))
        .route("/api/attendance", get(handlers::list_attendance))
        .route("/api/attendance/punch", post(handlers::attendance_punch))
        .route("/api/attendance/feed", get(handlers::attendance_feed))
        .route("/api/leave", get(handlers::list_leave_requests).post(handlers::leave_request))
        .route("/api/leave/:id/manager-decision", post(handlers::manager_decide_leave))
        .route("/api/leave/:id/hr-decision", post(handlers::hr_decide_leave))
        .route("/api/payroll/calculate", post(handlers::calculate_payroll))
        .route("/api/payroll/run", post(handlers::run_payroll))
        .route("/api/payroll/:run_id/edi", get(handlers::export_edi))
        .route("/api/hr-definitions", get(handlers::list_hr_definitions).post(handlers::create_hr_definition))
        .route("/api/administration/registrations", get(handlers::list_registrations));

    let app = Router::new()
        .merge(api)
        .merge(SwaggerUi::new("/api/docs").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
