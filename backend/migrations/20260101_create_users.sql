CREATE TABLE IF NOT EXISTS companies (
  id UUID PRIMARY KEY,
  name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS branches (
  id UUID PRIMARY KEY,
  company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  name TEXT NOT NULL UNIQUE,
  municipality TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS users (
  id UUID PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  role TEXT NOT NULL,
  branch_id UUID REFERENCES branches(id),
  is_active BOOLEAN NOT NULL DEFAULT true,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS employees (
  id UUID PRIMARY KEY,
  branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE RESTRICT,
  department TEXT,
  job_position TEXT NOT NULL,
  name TEXT NOT NULL,
  surname TEXT NOT NULL,
  birthdate DATE NOT NULL,
  country TEXT NOT NULL,
  personal_id TEXT NOT NULL UNIQUE,
  work_id TEXT NOT NULL UNIQUE,
  address TEXT NOT NULL,
  municipality TEXT NOT NULL,
  tel TEXT NOT NULL,
  official_email TEXT NOT NULL UNIQUE,
  employment_date DATE NOT NULL,
  marital_status TEXT NOT NULL,
  education TEXT NOT NULL,
  emergency_contact TEXT NOT NULL,
  family_connection TEXT NOT NULL,
  emergency_phone TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS contracts (
  id UUID PRIMARY KEY,
  employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
  contract_type TEXT NOT NULL,
  start_date DATE NOT NULL,
  end_date DATE,
  base_salary_eur NUMERIC(12,2) NOT NULL DEFAULT 0,
  coefficient NUMERIC(8,4) NOT NULL DEFAULT 1,
  status TEXT NOT NULL DEFAULT 'active'
);

CREATE TABLE IF NOT EXISTS attendance (
  id UUID PRIMARY KEY,
  employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
  branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE RESTRICT,
  click_type TEXT NOT NULL,
  happened_at TIMESTAMPTZ NOT NULL,
  camera_photo_ref TEXT,
  note TEXT,
  is_manual_correction BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS leave_requests (
  id UUID PRIMARY KEY,
  employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
  leave_type TEXT NOT NULL,
  start_date DATE NOT NULL,
  end_date DATE NOT NULL,
  status TEXT NOT NULL,
  manager_comment TEXT,
  hr_comment TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS payroll_runs (
  id UUID PRIMARY KEY,
  period_label TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS payroll_items (
  id UUID PRIMARY KEY,
  run_id UUID NOT NULL REFERENCES payroll_runs(id) ON DELETE CASCADE,
  employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
  gross_total NUMERIC(12,2) NOT NULL,
  net_total NUMERIC(12,2) NOT NULL,
  edi_line TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS salary_elements (
  id UUID PRIMARY KEY,
  employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
  element_name TEXT NOT NULL,
  amount NUMERIC(12,2) NOT NULL,
  period_label TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS hr_definitions (
  id UUID PRIMARY KEY,
  definition_type TEXT NOT NULL,
  key TEXT NOT NULL,
  value TEXT NOT NULL,
  UNIQUE(definition_type, key)
);

CREATE TABLE IF NOT EXISTS registrations (
  id UUID PRIMARY KEY,
  registration_type TEXT NOT NULL,
  value TEXT NOT NULL,
  UNIQUE(registration_type, value)
);

CREATE TABLE IF NOT EXISTS public_holidays (
  id UUID PRIMARY KEY,
  name TEXT NOT NULL,
  holiday_date DATE NOT NULL,
  UNIQUE(holiday_date, name)
);

CREATE TABLE IF NOT EXISTS audit_logs (
  id UUID PRIMARY KEY,
  actor_user_id UUID NOT NULL,
  action TEXT NOT NULL,
  details TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
