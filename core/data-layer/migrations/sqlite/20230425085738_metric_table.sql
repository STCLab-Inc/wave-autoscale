CREATE TABLE metric (
  db_id TEXT PRIMARY KEY,
  id TEXT UNIQUE,
  metric_kind TEXT NOT NULL,
  metadata TEXT,
  created_at TEXT,
  updated_at TEXT
);