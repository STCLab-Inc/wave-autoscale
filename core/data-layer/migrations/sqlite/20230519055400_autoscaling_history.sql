-- Add migration script here
CREATE TABLE autoscaling_history (
  id TEXT PRIMARY KEY,
  plan_db_id TEXT,
  plan_id TEXT,
  plan_item_json TEXT,
  metric_values_json TEXT,
  metadata_values_json TEXT,
  fail_message TEXT,
  created_at TEXT,
  FOREIGN KEY (plan_db_id) REFERENCES plan (db_id)
);