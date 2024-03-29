export const ISTIO_RETRY_CODE = `kind: ScalingComponent
id: k8s_patch
component_kind: kubernetes-json-patch
---
kind: Metric
id: prometheus_node_exporter_mem_usage
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
            "100 * (1 - ((avg_over_time(node_memory_MemFree_bytes[1m]) + avg_over_time(node_memory_Cached_bytes[1m]) + avg_over_time(node_memory_Buffers_bytes[1m])) / avg_over_time(node_memory_MemTotal_bytes[1m])))",
          ]
  transforms:
    my_transforms_id_1:
      inputs: ["my_source_id_1"]
      type: remap
      source: |-
        . = parse_json!(.message)
        tally = []
        for_each(array!(.data.result)) -> |_index, value| {
            tally = push(tally, {"name": "mem_usage", "tags": value.metric, "timestamp": value.value[0], "gauge": {"value" : to_float!(value.value[1])}})
        }
        tally
        . = tally
  sinks:
    my_sinks_id:
      type: wave-autoscale
      inputs: ["my_transforms_id_1"]
---
kind: ScalingPlan
id: scaling_plan_istio_retry
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
        metric_id: 'prometheus_node_exporter_mem_usage',
        name: 'mem_usage',
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
        # maximum of 3 retries and each with a 2 second timeout.
        json_patch:
          - op: add
            path: /spec/http/1/retries
            value: { "attempts": 3, "perTryTimeout": "2s" }
  - id: plan-scale-in
    description: "Scale in if CPU Utilization is less than 30."
    # JavaScript expression that returns a boolean value.
    expression: >
      get({
        metric_id: 'prometheus_node_exporter_mem_usage',
        name: 'mem_usage',
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
        # maximum of 1 retries and each with a 5 second timeout.
        json_patch:
          - op: add
            path: /http/1/retries
            value: { "attempts": 1, "perTryTimeout": "5s" }
`;
