export const DEPLOYMENT_REPLICAS_CODE = `kind: ScalingComponent
id: k8s_deployment
component_kind: kubernetes-deployment
enabled: false
metadata:
  api_server_endpoint: http://localhost:8080
  namespace: deployment-namespace
  name: deployment-name
---
# Metrics for the example above
kind: Metric
id: wa_metrics_generator
collector: wa-generator
enabled: false
metadata:
  pattern: [10, 20, 30, 40, 50, 60, 70, 80, 90, 100]
  gap_seconds: 10
---
kind: ScalingPlan
id: scaling_plan_k8s_deployment_scaling
metadata:
  title: "Scaling Plan for K8S Deployment Scaling - deployment replicas"
  cool_down: 60 # seconds
  interval: 10000 # milliseconds
enabled: false
plans:
  - id: plan-scale-out
    description: "Scale out if the metrics count is greater than 100 for 5 seconds"
    expression: >
      get({
        metric_id: 'wa_metrics_generator',
        stats: 'max',
        period_sec: 5
      }) >= 100
    # Higher priority values will be checked first.
    priority: 2
    scaling_components:
      - component_id: k8s_deployment
        # Available variables in the replicas expression:
        # - $replicas: current number of replicas
        # - $unavailable_replicas: current number of unavailable replicas
        # - $available_replicas: current number of available replicas
        # - $ready_replicas: current number of ready replicas
        # - $updated_replicas: current number of updated replicas
        replicas: $replicas + 3
  - id: plan-scale-in
    description: "Scale in if the metrics count is less than 50 for 5 seconds"
    # JavaScript expression that returns a boolean value.
    expression: >
      get({
        metric_id: 'wa_metrics_generator',
        stats: 'max',
        period_sec: 5
      }) < 50
    priority: 1
    scaling_components:
      - component_id: k8s_deployment
        replicas: 1`;
