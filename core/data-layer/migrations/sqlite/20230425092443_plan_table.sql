CREATE TABLE
  plan (
    db_id TEXT PRIMARY KEY,
    id TEXT UNIQUE,
    title TEXT,
    priority INTEGER,
    plans TEXT
  );