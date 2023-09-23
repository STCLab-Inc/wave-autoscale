-- Add migration script here
CREATE TABLE source_metrics (
    id TEXT PRIMARY KEY,
    collector TEXT NOT NULL,
    metric_id TEXT NOT NULL,
    json_value TEXT NOT NULL
);