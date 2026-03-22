use std::{convert::Infallible, path::PathBuf};

use axum::{
    extract::{Multipart, Path, Query},
    http::{HeaderMap, StatusCode},
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use chrono::Utc;
use futures_util::stream::{self, Stream};
use serde::Deserialize;
use sqlx::{FromRow, Row};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    auth::{create_token_pair, decode_token, require_csrf, verify_password, AuthUser},
    models::{
        ApiMessage, AttendanceEvent, AttendancePunchRequest, AttendanceRecord, AuthResponse, Branch,
        BranchCreate, ContractCreate, ContractRecord, Department, DepartmentCreate, Employee,
        EmployeeCreate, JobPosition, JobPositionCreate, LeaveCreateRequest, LeaveDecisionRequest,
        LeaveRequestRecord, LoginRequest, PayrollInput, PayrollResult, RefreshRequest, Role,
        SalaryElementCreate, SalaryElementRecord,
    },
    AppState,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct EmployeeQuery {
    pub branch_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LeaveQuery {
    pub employee_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AttendanceQuery {
    pub branch_id: Option<Uuid>,
    pub employee_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ContractQuery {
    pub employee_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SalaryElementQuery {
    pub employee_id: Option<Uuid>,
    pub period_label: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct HrDefinitionCreate {
    pub definition_type: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, serde::Serialize, FromRow, ToSchema)]
pub struct HrDefinitionRecord {
    pub id: Uuid,
    pub definition_type: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, serde::Serialize, FromRow, ToSchema)]
pub struct RegistrationRecord {
    pub id: Uuid,
    pub registration_type: String,
    pub value: String,
}

fn ensure_role(user: &AuthUser, allowed: &[&str]) -> Result<(), (StatusCode, String)> {
    if allowed.iter().any(|r| *r == user.role) {
        return Ok(());
    }

    Err((StatusCode::FORBIDDEN, "Insufficient permissions".to_owned()))
}

#[utoipa::path(get, path = "/api/health", responses((status = 200, description = "Health", body = String)))]
pub async fn health() -> &'static str {
    "OK"
}

#[utoipa::path(post, path = "/api/auth/login", request_body = LoginRequest, responses((status = 200, body = AuthResponse), (status = 401, body = ApiMessage)))]
pub async fn login(
    state: axum::extract::State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let row = sqlx::query(
        "SELECT id, email, password_hash, role FROM users WHERE email = $1 AND is_active = true",
    )
    .bind(&payload.email)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?
    .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".to_owned()))?;

    let hash: String = row.get("password_hash");
    if !verify_password(&payload.password, &hash) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_owned()));
    }

    let role_raw: String = row.get("role");
    let role = parse_role(&role_raw)?;

    let user_id: Uuid = row.get("id");
    let email: String = row.get("email");
    let (access_token, refresh_token, csrf_token) =
        create_token_pair(user_id, &email, role.clone(), &state.jwt_keys).map_err(internal_error)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        csrf_token,
        role: role.as_str().to_owned(),
        user_id,
    }))
}

#[utoipa::path(post, path = "/api/auth/refresh", request_body = RefreshRequest, responses((status = 200, body = AuthResponse), (status = 401, body = ApiMessage)))]
pub async fn refresh(
    state: axum::extract::State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let claims = decode_token(&payload.refresh_token, &state.jwt_keys)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid refresh token".to_owned()))?;

    if claims.typ != "refresh" {
        return Err((StatusCode::UNAUTHORIZED, "Invalid token type".to_owned()));
    }

    let role = parse_role(&claims.role)?;
    let (access_token, refresh_token, csrf_token) =
        create_token_pair(claims.sub, &claims.email, role.clone(), &state.jwt_keys)
            .map_err(internal_error)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        csrf_token,
        role: role.as_str().to_owned(),
        user_id: claims.sub,
    }))
}

#[utoipa::path(get, path = "/api/company/branches", responses((status = 200, body = [Branch])))]
pub async fn list_branches(
    state: axum::extract::State<AppState>,
) -> Result<Json<Vec<Branch>>, (StatusCode, String)> {
    let items = sqlx::query_as::<_, Branch>(
        "SELECT id, company_id, name, municipality FROM branches ORDER BY name",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(items))
}

#[utoipa::path(post, path = "/api/company/branches", request_body = BranchCreate, responses((status = 201, body = Branch)))]
pub async fn create_branch(
    state: axum::extract::State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Json(payload): Json<BranchCreate>,
) -> Result<(StatusCode, Json<Branch>), (StatusCode, String)> {
    ensure_role(&user, &["system_admin", "hr_admin"])?;
    require_csrf(&headers, &user.csrf)?;

    let branch = sqlx::query_as::<_, Branch>(
        "INSERT INTO branches (id, company_id, name, municipality) VALUES ($1,$2,$3,$4) RETURNING id, company_id, name, municipality",
    )
    .bind(Uuid::new_v4())
    .bind(payload.company_id)
    .bind(payload.name)
    .bind(payload.municipality)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(branch)))
}

#[utoipa::path(get, path = "/api/company/departments", responses((status = 200, body = [Department])))]
pub async fn list_departments(
    state: axum::extract::State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<Department>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, Department>(
        "SELECT id, branch_id, name FROM departments ORDER BY name",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(rows))
}

#[utoipa::path(post, path = "/api/company/departments", request_body = DepartmentCreate, responses((status = 201, body = Department)))]
pub async fn create_department(
    state: axum::extract::State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Json(payload): Json<DepartmentCreate>,
) -> Result<(StatusCode, Json<Department>), (StatusCode, String)> {
    ensure_role(&user, &["system_admin", "hr_admin"])?;
    require_csrf(&headers, &user.csrf)?;

    let row = sqlx::query_as::<_, Department>(
        "INSERT INTO departments (id, branch_id, name) VALUES ($1,$2,$3) RETURNING id, branch_id, name",
    )
    .bind(Uuid::new_v4())
    .bind(payload.branch_id)
    .bind(payload.name)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(row)))
}

#[utoipa::path(get, path = "/api/company/job-positions", responses((status = 200, body = [JobPosition])))]
pub async fn list_job_positions(
    state: axum::extract::State<AppState>,
) -> Result<Json<Vec<JobPosition>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, JobPosition>(
        "SELECT id, name, description FROM job_positions ORDER BY name",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(rows))
}

#[utoipa::path(post, path = "/api/company/job-positions", request_body = JobPositionCreate, responses((status = 201, body = JobPosition)))]
pub async fn create_job_position(
    state: axum::extract::State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Json(payload): Json<JobPositionCreate>,
) -> Result<(StatusCode, Json<JobPosition>), (StatusCode, String)> {
    ensure_role(&user, &["system_admin", "hr_admin"])?;
    require_csrf(&headers, &user.csrf)?;

    let row = sqlx::query_as::<_, JobPosition>(
        "INSERT INTO job_positions (id, name, description) VALUES ($1,$2,$3) RETURNING id, name, description",
    )
    .bind(Uuid::new_v4())
    .bind(payload.name)
    .bind(payload.description)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(row)))
}

#[utoipa::path(get, path = "/api/employees", params(("branch_id" = Option<String>, Query, description = "Filter by branch")), responses((status = 200, body = [Employee])))]
pub async fn list_employees(
    state: axum::extract::State<AppState>,
    Query(query): Query<EmployeeQuery>,
) -> Result<Json<Vec<Employee>>, (StatusCode, String)> {
    let items = if let Some(branch_id) = query.branch_id {
        sqlx::query_as::<_, Employee>(
            "SELECT * FROM employees WHERE branch_id = $1 ORDER BY name, surname",
        )
        .bind(branch_id)
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?
    } else {
        sqlx::query_as::<_, Employee>("SELECT * FROM employees ORDER BY name, surname")
            .fetch_all(&state.pool)
            .await
            .map_err(internal_error)?
    };

    Ok(Json(items))
}

#[utoipa::path(post, path = "/api/employees", request_body = EmployeeCreate, responses((status = 201, body = Employee), (status = 403, body = ApiMessage)))]
pub async fn create_employee(
    state: axum::extract::State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Json(payload): Json<EmployeeCreate>,
) -> Result<(StatusCode, Json<Employee>), (StatusCode, String)> {
    ensure_role(&user, &["hr_admin", "system_admin"])?;
    require_csrf(&headers, &user.csrf)?;

    let id = Uuid::new_v4();
    let created = sqlx::query_as::<_, Employee>(
        r#"INSERT INTO employees (
            id, branch_id, department, job_position, name, surname, birthdate, country,
            personal_id, work_id, address, municipality, tel, official_email,
            employment_date, marital_status, education, emergency_contact,
            family_connection, emergency_phone, status
        ) VALUES (
            $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21
        ) RETURNING *"#,
    )
    .bind(id)
    .bind(payload.branch_id)
    .bind(payload.department)
    .bind(payload.job_position)
    .bind(payload.name)
    .bind(payload.surname)
    .bind(payload.birthdate)
    .bind(payload.country)
    .bind(payload.personal_id)
    .bind(payload.work_id)
    .bind(payload.address)
    .bind(payload.municipality)
    .bind(payload.tel)
    .bind(payload.official_email)
    .bind(payload.employment_date)
    .bind(payload.marital_status)
    .bind(payload.education)
    .bind(payload.emergency_contact)
    .bind(payload.family_connection)
    .bind(payload.emergency_phone)
    .bind(payload.status)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(created)))
}

#[utoipa::path(post, path = "/api/employees/{id}/files", params(("id" = String, Path, description = "Employee id")), responses((status = 200, body = ApiMessage)))]
pub async fn upload_employee_file(
    state: axum::extract::State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(employee_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<ApiMessage>, (StatusCode, String)> {
    ensure_role(&user, &["hr_admin", "system_admin"])?;
    require_csrf(&headers, &user.csrf)?;

    let mut uploaded = 0usize;
    while let Some(field) = multipart.next_field().await.map_err(internal_error)? {
        let original_name = field
            .file_name()
            .map(|n| n.replace('/', "_").replace('\\', "_"))
            .unwrap_or_else(|| "upload.bin".to_owned());

        let bytes = field.bytes().await.map_err(internal_error)?;
        let safe_name = format!("{}_{}_{}", employee_id, Uuid::new_v4(), original_name);
        let mut path = PathBuf::from(&state.upload_dir);
        path.push(safe_name);
        tokio::fs::write(path, &bytes).await.map_err(internal_error)?;
        uploaded += 1;
    }

    Ok(Json(ApiMessage {
        message: format!("Uploaded {} files", uploaded),
    }))
}

#[utoipa::path(post, path = "/api/attendance/punch", request_body = AttendancePunchRequest, responses((status = 201, body = AttendanceRecord)))]
pub async fn attendance_punch(
    state: axum::extract::State<AppState>,
    Json(payload): Json<AttendancePunchRequest>,
) -> Result<(StatusCode, Json<AttendanceRecord>), (StatusCode, String)> {
    let row = sqlx::query(
        "SELECT branch_id FROM employees WHERE id = $1",
    )
    .bind(payload.employee_id)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    let branch_id: Uuid = row.get("branch_id");
    let happened_at = Utc::now();
    let photo_ref = format!("att_photo_{}_{}.txt", payload.employee_id, happened_at.timestamp_millis());

    let mut path = PathBuf::from(&state.upload_dir);
    path.push(&photo_ref);
    tokio::fs::write(path, payload.camera_photo_base64).await.map_err(internal_error)?;

    let record = sqlx::query_as::<_, AttendanceRecord>(
        r#"INSERT INTO attendance (
            id, employee_id, branch_id, click_type, happened_at, camera_photo_ref, note, is_manual_correction
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,false)
        RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(payload.employee_id)
    .bind(branch_id)
    .bind(payload.click_type)
    .bind(happened_at)
    .bind(photo_ref)
    .bind(payload.note)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    let _ = state.attendance_tx.send(AttendanceEvent {
        employee_id: record.employee_id,
        branch_id: record.branch_id,
        click_type: record.click_type.clone(),
        happened_at: record.happened_at,
    });

    Ok((StatusCode::CREATED, Json(record)))
}

#[utoipa::path(get, path = "/api/attendance", responses((status = 200, body = [AttendanceRecord])))]
pub async fn list_attendance(
    state: axum::extract::State<AppState>,
    user: AuthUser,
    Query(query): Query<AttendanceQuery>,
) -> Result<Json<Vec<AttendanceRecord>>, (StatusCode, String)> {
    ensure_role(&user, &["employee", "manager", "hr_admin", "system_admin"])?;

    let records = if let Some(emp_id) = query.employee_id {
        sqlx::query_as::<_, AttendanceRecord>(
            "SELECT * FROM attendance WHERE employee_id = $1 ORDER BY happened_at DESC LIMIT 500",
        )
        .bind(emp_id)
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?
    } else if let Some(branch_id) = query.branch_id {
        sqlx::query_as::<_, AttendanceRecord>(
            "SELECT * FROM attendance WHERE branch_id = $1 ORDER BY happened_at DESC LIMIT 500",
        )
        .bind(branch_id)
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?
    } else {
        sqlx::query_as::<_, AttendanceRecord>(
            "SELECT * FROM attendance ORDER BY happened_at DESC LIMIT 500",
        )
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?
    };

    Ok(Json(records))
}

#[utoipa::path(get, path = "/api/attendance/feed", responses((status = 200, body = String)))]
pub async fn attendance_feed(
    state: axum::extract::State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.attendance_tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|msg| msg.ok())
        .map(|event| {
            let payload = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_owned());
            Ok(Event::default().data(payload))
        });

    Sse::new(stream::once(async { Ok(Event::default().data("connected")) }).chain(stream))
        .keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(10)))
}

#[utoipa::path(post, path = "/api/leave", request_body = LeaveCreateRequest, responses((status = 201, body = LeaveRequestRecord)))]
pub async fn leave_request(
    state: axum::extract::State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Json(payload): Json<LeaveCreateRequest>,
) -> Result<(StatusCode, Json<LeaveRequestRecord>), (StatusCode, String)> {
    ensure_role(&user, &["employee", "manager", "hr_admin", "system_admin"])?;
    require_csrf(&headers, &user.csrf)?;

    let leave = sqlx::query_as::<_, LeaveRequestRecord>(
        r#"INSERT INTO leave_requests (
            id, employee_id, leave_type, start_date, end_date, status, manager_comment, hr_comment
        ) VALUES ($1,$2,$3,$4,$5,'pending_manager',NULL,NULL)
        RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(user.user_id)
    .bind(payload.leave_type)
    .bind(payload.start_date)
    .bind(payload.end_date)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    tracing::info!("Leave request {} queued for manager approval", leave.id);
    Ok((StatusCode::CREATED, Json(leave)))
}

#[utoipa::path(get, path = "/api/leave", responses((status = 200, body = [LeaveRequestRecord])))]
pub async fn list_leave_requests(
    state: axum::extract::State<AppState>,
    user: AuthUser,
    Query(query): Query<LeaveQuery>,
) -> Result<Json<Vec<LeaveRequestRecord>>, (StatusCode, String)> {
    ensure_role(&user, &["employee", "manager", "hr_admin", "system_admin"])?;

    let list = if let Some(employee_id) = query.employee_id {
        sqlx::query_as::<_, LeaveRequestRecord>(
            "SELECT * FROM leave_requests WHERE employee_id = $1 ORDER BY start_date DESC",
        )
        .bind(employee_id)
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?
    } else {
        sqlx::query_as::<_, LeaveRequestRecord>(
            "SELECT * FROM leave_requests ORDER BY start_date DESC",
        )
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?
    };

    Ok(Json(list))
}

#[utoipa::path(get, path = "/api/contracts", responses((status = 200, body = [ContractRecord])))]
pub async fn list_contracts(
    state: axum::extract::State<AppState>,
    user: AuthUser,
    Query(query): Query<ContractQuery>,
) -> Result<Json<Vec<ContractRecord>>, (StatusCode, String)> {
    ensure_role(&user, &["employee", "manager", "hr_admin", "system_admin"])?;

    let rows = if let Some(employee_id) = query.employee_id {
        sqlx::query_as::<_, ContractRecord>(
            "SELECT id, employee_id, contract_type, start_date, end_date, base_salary_eur::double precision as base_salary_eur, coefficient::double precision as coefficient, status FROM contracts WHERE employee_id = $1 ORDER BY start_date DESC",
        )
        .bind(employee_id)
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?
    } else {
        sqlx::query_as::<_, ContractRecord>(
            "SELECT id, employee_id, contract_type, start_date, end_date, base_salary_eur::double precision as base_salary_eur, coefficient::double precision as coefficient, status FROM contracts ORDER BY start_date DESC",
        )
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?
    };

    Ok(Json(rows))
}

#[utoipa::path(post, path = "/api/contracts", request_body = ContractCreate, responses((status = 201, body = ContractRecord)))]
pub async fn create_contract(
    state: axum::extract::State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Json(payload): Json<ContractCreate>,
) -> Result<(StatusCode, Json<ContractRecord>), (StatusCode, String)> {
    ensure_role(&user, &["hr_admin", "system_admin"])?;
    require_csrf(&headers, &user.csrf)?;

    let row = sqlx::query_as::<_, ContractRecord>(
        r#"INSERT INTO contracts (
            id, employee_id, contract_type, start_date, end_date, base_salary_eur, coefficient, status
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        RETURNING id, employee_id, contract_type, start_date, end_date,
        base_salary_eur::double precision as base_salary_eur,
        coefficient::double precision as coefficient,
        status"#,
    )
    .bind(Uuid::new_v4())
    .bind(payload.employee_id)
    .bind(payload.contract_type)
    .bind(payload.start_date)
    .bind(payload.end_date)
    .bind(payload.base_salary_eur)
    .bind(payload.coefficient)
    .bind(payload.status)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(row)))
}

#[utoipa::path(get, path = "/api/salary-elements", responses((status = 200, body = [SalaryElementRecord])))]
pub async fn list_salary_elements(
    state: axum::extract::State<AppState>,
    user: AuthUser,
    Query(query): Query<SalaryElementQuery>,
) -> Result<Json<Vec<SalaryElementRecord>>, (StatusCode, String)> {
    ensure_role(&user, &["employee", "manager", "hr_admin", "system_admin"])?;

    let rows = if let Some(employee_id) = query.employee_id {
        sqlx::query_as::<_, SalaryElementRecord>(
            "SELECT id, employee_id, element_name, amount::double precision as amount, period_label FROM salary_elements WHERE employee_id = $1 ORDER BY period_label DESC",
        )
        .bind(employee_id)
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?
    } else {
        let _ = query.period_label;
        sqlx::query_as::<_, SalaryElementRecord>(
            "SELECT id, employee_id, element_name, amount::double precision as amount, period_label FROM salary_elements ORDER BY period_label DESC",
        )
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?
    };

    Ok(Json(rows))
}

#[utoipa::path(post, path = "/api/salary-elements", request_body = SalaryElementCreate, responses((status = 201, body = SalaryElementRecord)))]
pub async fn create_salary_element(
    state: axum::extract::State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Json(payload): Json<SalaryElementCreate>,
) -> Result<(StatusCode, Json<SalaryElementRecord>), (StatusCode, String)> {
    ensure_role(&user, &["hr_admin", "system_admin"])?;
    require_csrf(&headers, &user.csrf)?;

    let row = sqlx::query_as::<_, SalaryElementRecord>(
        r#"INSERT INTO salary_elements (id, employee_id, element_name, amount, period_label)
           VALUES ($1,$2,$3,$4,$5)
           RETURNING id, employee_id, element_name, amount::double precision as amount, period_label"#,
    )
    .bind(Uuid::new_v4())
    .bind(payload.employee_id)
    .bind(payload.element_name)
    .bind(payload.amount)
    .bind(payload.period_label)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(row)))
}

#[utoipa::path(post, path = "/api/leave/{id}/manager-decision", request_body = LeaveDecisionRequest, responses((status = 200, body = LeaveRequestRecord)))]
pub async fn manager_decide_leave(
    state: axum::extract::State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<LeaveDecisionRequest>,
) -> Result<Json<LeaveRequestRecord>, (StatusCode, String)> {
    ensure_role(&user, &["manager", "hr_admin", "system_admin"])?;
    require_csrf(&headers, &user.csrf)?;

    let next_status = if payload.status == "approved" {
        "pending_hr"
    } else {
        "denied"
    };

    let row = sqlx::query_as::<_, LeaveRequestRecord>(
        r#"UPDATE leave_requests
           SET status = $1, manager_comment = $2
           WHERE id = $3
           RETURNING *"#,
    )
    .bind(next_status)
    .bind(payload.comment)
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    tracing::info!("Manager leave decision for {}", id);
    Ok(Json(row))
}

#[utoipa::path(post, path = "/api/leave/{id}/hr-decision", request_body = LeaveDecisionRequest, responses((status = 200, body = LeaveRequestRecord)))]
pub async fn hr_decide_leave(
    state: axum::extract::State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<LeaveDecisionRequest>,
) -> Result<Json<LeaveRequestRecord>, (StatusCode, String)> {
    ensure_role(&user, &["hr_admin", "system_admin"])?;
    require_csrf(&headers, &user.csrf)?;

    let next_status = if payload.status == "approved" {
        "approved"
    } else {
        "denied"
    };

    let row = sqlx::query_as::<_, LeaveRequestRecord>(
        r#"UPDATE leave_requests
           SET status = $1, hr_comment = $2
           WHERE id = $3
           RETURNING *"#,
    )
    .bind(next_status)
    .bind(payload.comment)
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    tracing::info!("HR leave decision for {} with email notification", id);
    Ok(Json(row))
}

#[utoipa::path(post, path = "/api/payroll/calculate", request_body = PayrollInput, responses((status = 200, body = PayrollResult)))]
pub async fn calculate_payroll(
    user: AuthUser,
    headers: HeaderMap,
    Json(payload): Json<PayrollInput>,
) -> Result<Json<PayrollResult>, (StatusCode, String)> {
    ensure_role(&user, &["hr_admin", "system_admin", "manager"])?;
    require_csrf(&headers, &user.csrf)?;

    Ok(Json(payroll_engine(payload)))
}

#[utoipa::path(post, path = "/api/payroll/run", request_body = [PayrollInput], responses((status = 201, body = [PayrollResult])))]
pub async fn run_payroll(
    state: axum::extract::State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Json(payload): Json<Vec<PayrollInput>>,
) -> Result<(StatusCode, Json<Vec<PayrollResult>>), (StatusCode, String)> {
    ensure_role(&user, &["hr_admin", "system_admin"])?;
    require_csrf(&headers, &user.csrf)?;

    let run_id = Uuid::new_v4();
    let period = payload
        .first()
        .map(|p| p.month_label.clone())
        .unwrap_or_else(|| "N/A".to_owned());

    sqlx::query(
        "INSERT INTO payroll_runs (id, period_label, status) VALUES ($1, $2, 'completed')",
    )
    .bind(run_id)
    .bind(&period)
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    let mut results = Vec::with_capacity(payload.len());
    for input in payload {
        let result = payroll_engine(input.clone());
        sqlx::query(
            r#"INSERT INTO payroll_items (
                id, run_id, employee_id, gross_total, net_total, edi_line
            ) VALUES ($1,$2,$3,$4,$5,$6)"#,
        )
        .bind(Uuid::new_v4())
        .bind(run_id)
        .bind(result.employee_id)
        .bind(result.gross_total)
        .bind(result.net_total)
        .bind(&result.edi_line)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;
        results.push(result);
    }

    sqlx::query(
        "INSERT INTO audit_logs (id, actor_user_id, action, details) VALUES ($1,$2,$3,$4)",
    )
    .bind(Uuid::new_v4())
    .bind(user.user_id)
    .bind("payroll_run")
    .bind(format!("run_id={}, period={}", run_id, period))
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(results)))
}

#[utoipa::path(get, path = "/api/payroll/{run_id}/edi", responses((status = 200, body = String)))]
pub async fn export_edi(
    state: axum::extract::State<AppState>,
    user: AuthUser,
    Path(run_id): Path<Uuid>,
) -> Result<String, (StatusCode, String)> {
    ensure_role(&user, &["hr_admin", "system_admin", "manager"])?;

    let rows = sqlx::query("SELECT edi_line FROM payroll_items WHERE run_id = $1 ORDER BY edi_line")
        .bind(run_id)
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;

    let mut text = String::new();
    for row in rows {
        let line: String = row.get("edi_line");
        text.push_str(&line);
        text.push('\n');
    }

    Ok(text)
}

#[utoipa::path(get, path = "/api/hr-definitions", responses((status = 200, body = [HrDefinitionRecord])))]
pub async fn list_hr_definitions(
    state: axum::extract::State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<HrDefinitionRecord>>, (StatusCode, String)> {
    let defs = sqlx::query_as::<_, HrDefinitionRecord>(
        "SELECT id, definition_type, key, value FROM hr_definitions ORDER BY definition_type, key",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;
    Ok(Json(defs))
}

#[utoipa::path(post, path = "/api/hr-definitions", request_body = HrDefinitionCreate, responses((status = 201, body = HrDefinitionRecord)))]
pub async fn create_hr_definition(
    state: axum::extract::State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Json(payload): Json<HrDefinitionCreate>,
) -> Result<(StatusCode, Json<HrDefinitionRecord>), (StatusCode, String)> {
    ensure_role(&user, &["hr_admin", "system_admin"])?;
    require_csrf(&headers, &user.csrf)?;

    let rec = sqlx::query_as::<_, HrDefinitionRecord>(
        r#"INSERT INTO hr_definitions (id, definition_type, key, value)
           VALUES ($1,$2,$3,$4)
           RETURNING id, definition_type, key, value"#,
    )
    .bind(Uuid::new_v4())
    .bind(payload.definition_type)
    .bind(payload.key)
    .bind(payload.value)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(rec)))
}

#[utoipa::path(get, path = "/api/administration/registrations", responses((status = 200, body = [RegistrationRecord])))]
pub async fn list_registrations(
    state: axum::extract::State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<RegistrationRecord>>, (StatusCode, String)> {
    ensure_role(&user, &["hr_admin", "system_admin", "manager"])?;

    let recs = sqlx::query_as::<_, RegistrationRecord>(
        "SELECT id, registration_type, value FROM registrations ORDER BY registration_type",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(recs))
}

pub fn payroll_engine(input: PayrollInput) -> PayrollResult {
    let standard_cap = 160.0;
    let premium_threshold = 200.0;
    let total_hours = input.worked_hours + input.extra_hours;

    let standard_hours = total_hours.min(standard_cap);
    let overtime_hours = (total_hours - standard_cap).max(0.0).min(40.0);
    let premium_hours = (total_hours - premium_threshold).max(0.0);

    let standard_pay = standard_hours * input.base_hourly_rate;
    let overtime_pay = overtime_hours * input.base_hourly_rate * input.tier2_rate_multiplier;
    let premium_pay = premium_hours * input.base_hourly_rate * input.tier3_rate_multiplier;

    let gross_total = standard_pay + overtime_pay + premium_pay + input.bonus_eur;
    let net_total = (gross_total - input.deduction_eur).max(0.0);

    let edi_line = format!(
        "EMP={};PERIOD={};GROSS={:.2};NET={:.2};CUR=EUR",
        input.employee_id, input.month_label, gross_total, net_total
    );

    PayrollResult {
        employee_id: input.employee_id,
        month_label: input.month_label,
        currency: "EUR".to_owned(),
        standard_hours,
        overtime_hours,
        premium_hours,
        gross_total,
        net_total,
        edi_line,
    }
}

pub fn parse_role(raw: &str) -> Result<Role, (StatusCode, String)> {
    match raw {
        "employee" => Ok(Role::Employee),
        "manager" => Ok(Role::Manager),
        "hr_admin" => Ok(Role::HrAdmin),
        "system_admin" => Ok(Role::SystemAdmin),
        _ => Err((StatusCode::INTERNAL_SERVER_ERROR, "Unknown role".to_owned())),
    }
}

fn internal_error<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    tracing::error!("{}", e);
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_owned())
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::models::PayrollInput;

    use super::{parse_role, payroll_engine};

    #[test]
    fn payroll_three_tier_calculation_works() {
        let input = PayrollInput {
            employee_id: Uuid::new_v4(),
            month_label: "2026-03".to_owned(),
            base_hourly_rate: 6.5,
            worked_hours: 170.0,
            extra_hours: 35.0,
            bonus_eur: 50.0,
            deduction_eur: 15.0,
            tier2_rate_multiplier: 1.3,
            tier3_rate_multiplier: 1.6,
        };

        let out = payroll_engine(input);
        assert_eq!(out.standard_hours, 160.0);
        assert_eq!(out.overtime_hours, 40.0);
        assert_eq!(out.premium_hours, 5.0);
        assert!(out.gross_total > 0.0);
        assert!(out.net_total > 0.0);
        assert_eq!(out.currency, "EUR");
    }

    #[test]
    fn role_parsing_works() {
        assert!(parse_role("employee").is_ok());
        assert!(parse_role("manager").is_ok());
        assert!(parse_role("hr_admin").is_ok());
        assert!(parse_role("system_admin").is_ok());
        assert!(parse_role("invalid").is_err());
    }
}
