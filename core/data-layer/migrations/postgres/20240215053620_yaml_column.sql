-- Add migration script here
ALTER TABLE plan
ADD COLUMN yaml TEXT;
ALTER TABLE metric
ADD COLUMN yaml TEXT;
ALTER TABLE scaling_component
ADD COLUMN yaml TEXT;