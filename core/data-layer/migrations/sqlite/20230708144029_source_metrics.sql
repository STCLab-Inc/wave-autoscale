-- Add migration script here
CREATE TABLE source_metrics (
    id TEXT PRIMARY KEY,
    collector INTEGER NOT NULL,
    metric_id INTEGER NOT NULL,
    json_value TEXT NOT NULL
);