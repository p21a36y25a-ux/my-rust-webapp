use chrono::NaiveDate;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    auth::hash_password,
    models::Role,
    AppState,
};

pub async fn seed_demo_data(state: &AppState) -> anyhow::Result<()> {
    let company_id = ensure_company(state).await?;
    let branch_prishtina = ensure_branch(state, company_id, "Prishtina", "Prishtine").await?;
    let branch_peja = ensure_branch(state, company_id, "Peja", "Peje").await?;
    let branch_prizreni = ensure_branch(state, company_id, "Prizreni", "Prizren").await?;

    ensure_job_positions(state).await?;
    ensure_hr_definitions(state).await?;
    ensure_registrations(state).await?;
    ensure_holidays(state).await?;

    ensure_demo_user(
        state,
        "system_admin@example.com",
        "Password123!",
        Role::SystemAdmin,
        Some(branch_prishtina),
    )
    .await?;
    ensure_demo_user(
        state,
        "hr_admin@example.com",
        "Password123!",
        Role::HrAdmin,
        Some(branch_prishtina),
    )
    .await?;
    ensure_demo_user(
        state,
        "manager_prishtina@example.com",
        "Password123!",
        Role::Manager,
        Some(branch_prishtina),
    )
    .await?;
    let employee_user = ensure_demo_user(
        state,
        "employee_01_prishtina@example.com",
        "Password123!",
        Role::Employee,
        Some(branch_prishtina),
    )
    .await?;

    ensure_demo_employee(
        state,
        employee_user,
        branch_prishtina,
        "Arta",
        "Krasniqi",
        "employee_01_prishtina@example.com",
        "Keshilltarë per Klient",
        "Prishtine",
    )
    .await?;

    let blendi_user = ensure_demo_user(
        state,
        "blendi.peja@example.com",
        "Password123!",
        Role::Employee,
        Some(branch_peja),
    )
    .await?;
    ensure_demo_employee(
        state,
        blendi_user,
        branch_peja,
        "Blendi",
        "Gashi",
        "blendi.peja@example.com",
        "Menaxher",
        "Peje",
    )
    .await?;

    let drita_user = ensure_demo_user(
        state,
        "drita.prizren@example.com",
        "Password123!",
        Role::Employee,
        Some(branch_prizreni),
    )
    .await?;
    ensure_demo_employee(
        state,
        drita_user,
        branch_prizreni,
        "Drita",
        "Bytyqi",
        "drita.prizren@example.com",
        "Inxhinier i Hidros",
        "Prizren",
    )
    .await?;

    Ok(())
}

async fn ensure_company(state: &AppState) -> anyhow::Result<Uuid> {
    let rec = sqlx::query("SELECT id FROM companies WHERE name = 'Demo Company LLC'")
        .fetch_optional(&state.pool)
        .await?;

    if let Some(row) = rec {
        return Ok(row.get("id"));
    }

    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO companies (id, name) VALUES ($1, 'Demo Company LLC')")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(id)
}

async fn ensure_branch(state: &AppState, company_id: Uuid, name: &str, municipality: &str) -> anyhow::Result<Uuid> {
    if let Some(row) = sqlx::query("SELECT id FROM branches WHERE name = $1")
        .bind(name)
        .fetch_optional(&state.pool)
        .await?
    {
        return Ok(row.get("id"));
    }

    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO branches (id, company_id, name, municipality) VALUES ($1,$2,$3,$4)")
        .bind(id)
        .bind(company_id)
        .bind(name)
        .bind(municipality)
        .execute(&state.pool)
        .await?;

    Ok(id)
}

async fn ensure_demo_user(
    state: &AppState,
    email: &str,
    password: &str,
    role: Role,
    branch_id: Option<Uuid>,
) -> anyhow::Result<Uuid> {
    if let Some(row) = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(&state.pool)
        .await?
    {
        return Ok(row.get("id"));
    }

    let id = Uuid::new_v4();
    let hash = hash_password(password)?;
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, role, branch_id, is_active) VALUES ($1,$2,$3,$4,$5,true)",
    )
    .bind(id)
    .bind(email)
    .bind(hash)
    .bind(role.as_str())
    .bind(branch_id)
    .execute(&state.pool)
    .await?;

    Ok(id)
}

async fn ensure_demo_employee(
    state: &AppState,
    id: Uuid,
    branch_id: Uuid,
    name: &str,
    surname: &str,
    email: &str,
    job_position: &str,
    municipality: &str,
) -> anyhow::Result<()> {
    if sqlx::query("SELECT id FROM employees WHERE official_email = $1")
        .bind(email)
        .fetch_optional(&state.pool)
        .await?
        .is_some()
    {
        return Ok(());
    }

    sqlx::query(
        r#"INSERT INTO employees (
            id, branch_id, department, job_position, name, surname, birthdate, country,
            personal_id, work_id, address, municipality, tel, official_email,
            employment_date, marital_status, education, emergency_contact,
            family_connection, emergency_phone, status
        ) VALUES (
            $1,$2,'General',$3,$4,$5,$6,'Kosova',$7,$8,'Rruga Kryesore',$9,'+38344111222',$10,
            $11,'Single','Bachelor','Prind','Prind','+38344111333','Aktiv'
        )"#,
    )
    .bind(id)
    .bind(branch_id)
    .bind(job_position)
    .bind(name)
    .bind(surname)
    .bind(NaiveDate::from_ymd_opt(1993, 5, 4).expect("valid date"))
    .bind(format!("PID-{}", &id.to_string()[..8]))
    .bind(format!("WID-{}", &id.to_string()[..6]))
    .bind(municipality)
    .bind(email)
    .bind(NaiveDate::from_ymd_opt(2022, 1, 10).expect("valid date"))
    .execute(&state.pool)
    .await?;

    Ok(())
}

async fn ensure_job_positions(state: &AppState) -> anyhow::Result<()> {
    let items = [
        "Menaxher",
        "Asistent menaxher",
        "Depoist Depo",
        "Keshilltarë per Klient",
        "Arkitekt",
        "Ndihmese",
        "Shtepiak",
        "Menaxher i Depos",
        "Kordinator I Shitjes",
        "Menaxher Importi",
        "Vozites",
        "Inxhinier i Hidros",
    ];

    for item in items {
        sqlx::query(
            "INSERT INTO hr_definitions (id, definition_type, key, value) VALUES ($1,'job_position',$2,$3) ON CONFLICT (definition_type, key) DO NOTHING",
        )
        .bind(Uuid::new_v4())
        .bind(item)
        .bind(item)
        .execute(&state.pool)
        .await?;
    }

    Ok(())
}

async fn ensure_hr_definitions(state: &AppState) -> anyhow::Result<()> {
    let rows = [
        ("employee_status", "Aktiv", "Active / Aktiv"),
        ("employee_status", "Pasiv", "Inactive / Pasiv"),
        ("contract_type", "Kohe_pacaktuar", "Indefinite"),
        ("contract_type", "Kohe_caktuar", "Fixed-term"),
        ("vacation_type", "Vjetor", "Annual"),
        ("vacation_type", "Mjekesor", "Medical"),
        ("probation_type", "3_muaj", "3 months"),
        ("calculation_type", "tiered_hourly", "Three-tier hourly calculation"),
        ("coefficient", "default", "1.00"),
        ("salary_element", "base", "Base salary"),
        ("salary_element", "bonus", "Bonus"),
    ];

    for (definition_type, key, value) in rows {
        sqlx::query(
            "INSERT INTO hr_definitions (id, definition_type, key, value) VALUES ($1,$2,$3,$4) ON CONFLICT (definition_type, key) DO NOTHING",
        )
        .bind(Uuid::new_v4())
        .bind(definition_type)
        .bind(key)
        .bind(value)
        .execute(&state.pool)
        .await?;
    }

    Ok(())
}

async fn ensure_registrations(state: &AppState) -> anyhow::Result<()> {
    let rows = [
        ("municipal", "Prishtine-REG-001"),
        ("state", "KS-STATE-987654"),
        ("bank", "XK051212012345678906"),
        ("marital_status", "Single"),
        ("marital_status", "Married"),
    ];

    for (t, value) in rows {
        sqlx::query(
            "INSERT INTO registrations (id, registration_type, value) VALUES ($1,$2,$3) ON CONFLICT (registration_type, value) DO NOTHING",
        )
        .bind(Uuid::new_v4())
        .bind(t)
        .bind(value)
        .execute(&state.pool)
        .await?;
    }

    Ok(())
}

async fn ensure_holidays(state: &AppState) -> anyhow::Result<()> {
    let rows = [
        ("Bajrami madh", NaiveDate::from_ymd_opt(2026, 3, 30).expect("valid date")),
        ("Bajrami vogel", NaiveDate::from_ymd_opt(2026, 6, 18).expect("valid date")),
        ("Krishtlindjet", NaiveDate::from_ymd_opt(2026, 12, 25).expect("valid date")),
        ("Viti Ri", NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date")),
        ("Dita e Pavarësis", NaiveDate::from_ymd_opt(2026, 2, 17).expect("valid date")),
    ];

    for (name, holiday_date) in rows {
        sqlx::query(
            "INSERT INTO public_holidays (id, name, holiday_date) VALUES ($1,$2,$3) ON CONFLICT (holiday_date, name) DO NOTHING",
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .bind(holiday_date)
        .execute(&state.pool)
        .await?;
    }

    Ok(())
}
