export const VECTOR_POSTGRES_CODE = `# We obtain the Istio request duration metric value from Prometheus through the vector agent.
kind: Metric
id: postgresql
collector: vector
metadata:
  sources:
    my_source_id_1:
      type: postgresql_metrics
      # If you use the Prometheus endpoint at '/api/v1/query' you will receive a value in JSON format.
      # Therefore, you need to use Vector's transforms to convert it to match the format of wave-autoscale.
      endpoints:
        - postgres://user:password@url/postgres
      namespace: postgresql
      scrape_interval_secs: 30
  sinks:
    my_sinks_id:
      # The 'sinks' type in Vector should be set to 'wave-autoscale'.
      type: wave-autoscale
      # For the 'inputs' value, you should basically provide the 'id' value from 'sources'.
      # If there are 'transforms', you can input the 'id' of the last transformed form.
      inputs: ["my_source_id_1"]
########### [ Saved Metric Data Example ] ############
# pg_stat_bgwriter_maxwritten_clean_total
# {
#   [{"name": "up","value": 1.0,"timestamp": "2023-10-31T08:57:56.126699Z"},{"name": "pg_stat_bgwriter_checkpoints_timed_total","value": 1783.0,"timestamp": "2023-10-31T08:57:56.124242Z"},{"name": "pg_stat_bgwriter_checkpoints_req_total","value": 10.0,"timestamp": "2023-10-31T08:57:56.124253Z"},{"name": "pg_stat_bgwriter_checkpoint_write_time_seconds_total","value": 322.164,"timestamp": "2023-10-31T08:57:56.124261Z"},{"name": "pg_stat_bgwriter_checkpoint_sync_time_seconds_total","value": 5.966,"timestamp": "2023-10-31T08:57:56.124264Z"},{"name": "pg_stat_bgwriter_buffers_checkpoint_total","value": 4136.0,"timestamp": "2023-10-31T08:57:56.124268Z"},{"name": "pg_stat_bgwriter_buffers_clean_total","value": 0.0,"timestamp": "2023-10-31T08:57:56.124278Z"},{"name": "pg_stat_bgwriter_maxwritten_clean_total","value": 0.0,"timestamp": "2023-10-31T08:57:56.124282Z"},{"name": "pg_stat_bgwriter_buffers_backend_total","value": 458.0,"timestamp": "2023-10-31T08:57:56.124285Z"},{"name": "pg_stat_bgwriter_buffers_backend_fsync_total","value": 0.0,"timestamp": "2023-10-31T08:57:56.124291Z"},{"name": "pg_stat_bgwriter_buffers_alloc_total","value": 9460.0,"timestamp": "2023-10-31T08:57:56.124296Z"},{"name": "pg_stat_bgwriter_stats_reset","value": 1687908327.0,"timestamp": "2023-10-31T08:57:56.124627Z"}]
# }
######################################################
`;
