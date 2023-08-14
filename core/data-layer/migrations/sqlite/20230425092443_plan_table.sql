CREATE TABLE
  plan (
    db_id TEXT PRIMARY KEY,
    id TEXT UNIQUE,
    title TEXT,
    priority INTEGER,
    interval INTEGER,
    metadata TEXT,
    plans TEXT,
    created_at TEXT,
    updated_at TEXT
  );