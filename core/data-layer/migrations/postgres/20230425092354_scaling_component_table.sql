CREATE TABLE scaling_component (
  db_id TEXT PRIMARY KEY,
  id TEXT UNIQUE,
  component_kind TEXT NOT NULL,
  metadata TEXT,
  enabled BOOLEAN,
  created_at timestamptz,
  updated_at timestamptz
);