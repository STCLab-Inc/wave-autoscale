CREATE TABLE
  plan (
    db_id TEXT PRIMARY KEY,
    id TEXT UNIQUE,
    priority INTEGER,
    metadata TEXT,
    plans TEXT,
    created_at TEXT,
    updated_at TEXT
  );