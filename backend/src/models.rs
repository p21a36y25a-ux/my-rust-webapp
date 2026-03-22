use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Employee,
    Manager,
    HrAdmin,
    SystemAdmin,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Employee => "employee",
            Self::Manager => "manager",
            Self::HrAdmin => "hr_admin",
            Self::SystemAdmin => "system_admin",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Branch {
    pub id: Uuid,
    pub company_id: Uuid,
    pub name: String,
    pub municipality: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BranchCreate {
    pub company_id: Uuid,
    pub name: String,
    pub municipality: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Department {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DepartmentCreate {
    pub branch_id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DepartmentUpdate {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct JobPosition {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JobPositionCreate {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JobPositionUpdate {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Employee {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub department: Option<String>,
    pub job_position: String,
    pub name: String,
    pub surname: String,
    pub birthdate: NaiveDate,
    pub country: String,
    pub personal_id: String,
    pub work_id: String,
    pub address: String,
    pub municipality: String,
    pub tel: String,
    pub official_email: String,
    pub employment_date: NaiveDate,
    pub marital_status: String,
    pub education: String,
    pub emergency_contact: String,
    pub family_connection: String,
    pub emergency_phone: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EmployeeCreate {
    pub branch_id: Uuid,
    pub department: Option<String>,
    pub job_position: String,
    pub name: String,
    pub surname: String,
    pub birthdate: NaiveDate,
    pub country: String,
    pub personal_id: String,
    pub work_id: String,
    pub address: String,
    pub municipality: String,
    pub tel: String,
    pub official_email: String,
    pub employment_date: NaiveDate,
    pub marital_status: String,
    pub education: String,
    pub emergency_contact: String,
    pub family_connection: String,
    pub emergency_phone: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub csrf_token: String,
    pub role: String,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AttendanceRecord {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub branch_id: Uuid,
    pub click_type: String,
    pub happened_at: DateTime<Utc>,
    pub camera_photo_ref: Option<String>,
    pub note: Option<String>,
    pub is_manual_correction: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttendancePunchRequest {
    pub employee_id: Uuid,
    pub click_type: String,
    pub camera_photo_base64: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct LeaveRequestRecord {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub leave_type: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status: String,
    pub manager_comment: Option<String>,
    pub hr_comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LeaveCreateRequest {
    pub leave_type: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ContractRecord {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub contract_type: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub base_salary_eur: f64,
    pub coefficient: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ContractCreate {
    pub employee_id: Uuid,
    pub contract_type: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub base_salary_eur: f64,
    pub coefficient: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ContractUpdate {
    pub contract_type: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub base_salary_eur: f64,
    pub coefficient: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct SalaryElementRecord {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub element_name: String,
    pub amount: f64,
    pub period_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SalaryElementCreate {
    pub employee_id: Uuid,
    pub element_name: String,
    pub amount: f64,
    pub period_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SalaryElementUpdate {
    pub element_name: String,
    pub amount: f64,
    pub period_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LeaveDecisionRequest {
    pub status: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PayrollInput {
    pub employee_id: Uuid,
    pub month_label: String,
    pub base_hourly_rate: f64,
    pub worked_hours: f64,
    pub extra_hours: f64,
    pub bonus_eur: f64,
    pub deduction_eur: f64,
    pub tier2_rate_multiplier: f64,
    pub tier3_rate_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PayrollResult {
    pub employee_id: Uuid,
    pub month_label: String,
    pub currency: String,
    pub standard_hours: f64,
    pub overtime_hours: f64,
    pub premium_hours: f64,
    pub gross_total: f64,
    pub net_total: f64,
    pub edi_line: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttendanceEvent {
    pub employee_id: Uuid,
    pub branch_id: Uuid,
    pub click_type: String,
    pub happened_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiMessage {
    pub message: String,
}
