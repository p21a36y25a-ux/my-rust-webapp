CREATE TABLE IF NOT EXISTS departments (
  id UUID PRIMARY KEY,
  branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  UNIQUE(branch_id, name)
);

CREATE TABLE IF NOT EXISTS job_positions (
  id UUID PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  description TEXT
);
