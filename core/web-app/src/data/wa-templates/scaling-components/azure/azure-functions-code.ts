export const AZURE_FUNCTIONS_CODE = `kind: ScalingComponent
id: scaling_component_azure_functions_app
component_kind: azure-functions-app
metadata:
  client_id: "{{ client_id }}"
  client_secret: "{{ client_secret }}"
  tenant_id: "{{ tenant_id }}"
  subscription_id: "{{ subscription_id }}"
  resource_group_name: "{{ resource_group_name }}"
  app_name: "{{ app_name }}"
---
kind: Metric
id: telegraf_prometheus_azure_functions_metrics
collector: telegraf
metadata:
  inputs:
    prometheus:
      - urls: [http://0.0.0.0:9090/metrics]
        period: 1m
        delay: 0s
        interval: 1m
        namepass: [nginx_connections_waiting] # This metric represents the wating number of connections that the nginx reserves. If this value remains high, it signifies that the nginx is using a significant amount of connections, which could trigger a scale-out.
  outputs:
    wave-autoscale: {}
---
kind: ScalingPlan
id: scaling_plan_azure_functions_app
title: scaling plan for azure functions app
  cool_down: 0 # seconds
  interval: 60000 # milliseconds
plans:
  # When the maximum wating number of connections is low (less than 0 or up to 10), the app configuration values decreases (scaling in), which reflects the low usage.
  # When the maximum wating number of connections is high (exceeding 100 or surpassing 1000), the app configuration values increases (scaling out) to meet the high demand.  
  - id: scaling_plan_azure_functions_app_scale_in_1
    description: scale-in instance count
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_prometheus_azure_functions_metrics',
        name: 'nginx_connections_waiting',        
        stats: 'max',
        period_sec: 60
      }) < 0
    # Higher priority values will be checked first.
    priority: 2
    scaling_components:
      - component_id: scaling_component_azure_functions_app
        min_instance_count: 1
        max_instance_count: 2
  - id: scaling_plan_azure_functions_app_scale_in_2
    description: scale-in instance count
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_prometheus_azure_functions_metrics',
        name: 'nginx_connections_waiting',        
        stats: 'max',
        period_sec: 60
      }) < 10
    # Higher priority values will be checked first.
    priority: 1
    scaling_components:
      - component_id: scaling_component_azure_functions_app
        min_instance_count: 5
        max_instance_count: 10
  - id: scaling_plan_azure_functions_app_scale_out_1
    description: scale-out instance count
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_prometheus_azure_functions_metrics',
        name: 'nginx_connections_waiting',        
        stats: 'max',
        period_sec: 60
      }) > 100
    # Higher priority values will be checked first.
    priority: 3
    scaling_components:
      - component_id: scaling_component_azure_functions_app
        min_instance_count: 10
        max_instance_count: 20
  - id: scaling_plan_azure_functions_app_scale_out_2
    description: scale-out instance count
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_prometheus_azure_functions_metrics',
        name: 'nginx_connections_waiting',        
        stats: 'max',
        period_sec: 60
      }) > 1000
    # Higher priority values will be checked first.
    priority: 4
    scaling_components:
      - component_id: scaling_component_azure_functions_app
        min_instance_count: 100
        max_instance_count: 200`;
