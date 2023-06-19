CREATE TABLE scaling_component (
  db_id TEXT PRIMARY KEY,
  id TEXT UNIQUE,
  component_kind TEXT NOT NULL,
  metadata TEXT,
  created_at TEXT,
  updated_at TEXT
);