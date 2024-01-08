CREATE TABLE metric (
  db_id TEXT PRIMARY KEY,
  id TEXT UNIQUE,
  collector TEXT,
  metadata TEXT,
  enabled BOOLEAN,
  created_at TEXT,
  updated_at TEXT
);