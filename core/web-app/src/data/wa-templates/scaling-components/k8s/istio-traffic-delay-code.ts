export const ISTIO_TRAFFIC_DELAY_CODE = `kind: ScalingComponent
id: k8s_patch
component_kind: kubernetes-json-patch
---
kind: Metric
id: prometheus_node_exporter_cpu_usage
collector: vector
metadata:
  sources:
    my_source_id_1:
      type: http_client
      endpoint: "http://localhost:9090/api/v1/query"
      scrape_interval_secs: 30
      query:
        "query":
          [
            '100 - (avg by (instance) (irate(node_cpu_seconds_total{mode="idle"}[1m])) * 100)',
          ]
  transforms:
    my_transforms_id_1:
      inputs: ["my_source_id_1"]
      type: remap
      source: |-
        . = parse_json!(.message)
        tally = []
        for_each(array!(.data.result)) -> |_index, value| {
            tally = push(tally, {"name": "cpu_usage", "tags": value.metric, "timestamp": value.value[0], "gauge": {"value" : to_float!(value.value[1])}})
        }
        tally
        . = tally
  sinks:
    my_sinks_id:
      type: wave-autoscale
      inputs: ["my_transforms_id_1"]
---
kind: ScalingPlan
id: scaling_plan_istio_traffic_delay
metadata:
  title: "Scaling Plan for K8S Patch Scaling - istio virtual service dely"
  cool_down: 30 # seconds
  interval: 10000 # milliseconds
plans:
  - id: plan-scale-out
    description: "Scale out if CPU Utilization is greater than 70%."
    # JavaScript expression that returns a boolean value.
    expression: >
      get({
        metric_id: 'prometheus_node_exporter_cpu_usage',
        name: 'cpu_usage',
        stats: 'max',
        period_sec: 100
      }) >= 70
    # Higher priority values will be checked first.
    priority: 2
    scaling_components:
      - component_id: k8s_patch
        namespace: istio-system
        name: istio-vs
        api_version: networking.istio.io/v1beta1
        kind: VirtualService
        # Kubernetes json patch
        json_patch:
          - op: add
            path: /spec/http/0/fault
            value: { "delay": { "fixedDelay": "10s" } }
          - op: add
            path: /spec/http/0/fault/delay/percentage
            value: { "value": 100 }
  - id: plan-scale-in
    description: "Scale in if CPU Utilization is less than 30."
    # JavaScript expression that returns a boolean value.
    expression: >
      get({
        metric_id: 'prometheus_node_exporter_cpu_usage',
        name: 'cpu_usage',
        stats: 'max',
        period_sec: 30
      }) < 30
    # Higher priority values will be checked first.
    priority: 1
    scaling_components:
      - component_id: k8s_patch
        namespace: istio-system
        name: istio-vs
        api_version: networking.istio.io/v1beta1
        kind: VirtualService
        # Kubernetes json patch
        json_patch:
          - op: add
            path: /spec/http/0/fault
            value: null
          # If you use remove, you may experience errors in autoscaling_history.
          # - op: remove
          #   path: /spec/http/0/fault
`;
