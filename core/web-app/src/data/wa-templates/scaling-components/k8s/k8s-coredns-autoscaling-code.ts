export const COREDNS_AUTOSCALING_CODE = `kind: ScalingComponent
id: k8s_deployment
component_kind: kubernetes-deployment
enabled: false
metadata:
  api_server_endpoint: http://localhost:8080
  namespace: kube-system
  name: coredns
---
kind: Metric
id: prometheus_coredns_request_5m
collector: vector
metadata:
  sources:
    my_source_id_1:
      type: http_client
      endpoint: "http://localhost:9090/api/v1/query"
      scrape_interval_secs: 30
      query:
        "query": ['rate(coredns_dns_requests_total{type=~"A|AAAA"}[5m])']
  transforms:
    my_transforms_id_1:
      inputs: ["my_source_id_1"]
      type: remap
      source: |-
        . = parse_json!(.message)
        tally = []
        for_each(array!(.data.result)) -> |_index, value| {
            tally = push(tally, {"name": "coredns_request", "tags": value.metric, "timestamp": value.value[0], "gauge": {"value" : to_float!(value.value[1])}})
        }
        tally
        . = tally
  sinks:
    my_sinks_id:
      type: wave-autoscale
      inputs: ["my_transforms_id_1"]
---
kind: ScalingPlan
id: scaling_plan_k8s_coredns_scaling
metadata:
  title: "Scaling Plan for K8S CoreDNS Scaling - scaling replicas"
  cool_down: 30 # seconds
  interval: 10000 # milliseconds
plans:
  - id: plan-scale-out
    description: "Scale out if CoreDNS request rate of increase is greater than 40%."
    # JavaScript expression that returns a boolean value.
    expression: >
      get({
        metric_id: 'prometheus_coredns_request_5m',
        name: 'coredns_request',
        stats: 'max',
        period_sec: 60
      }) >= 40
    # Higher priority values will be checked first.
    priority: 2
    scaling_components:
      - component_id: k8s_corddns
        replicas: $replicas + 1
  - id: plan-scale-in
    description: "Scale in if CoreDNS request rate of increase is less than 10%."
    # JavaScript expression that returns a boolean value.
    expression: >
      get({
        metric_id: 'prometheus_coredns_request_5m',
        name: 'coredns_request',
        stats: 'max',
        period_sec: 60
      }) < 10
    # Higher priority values will be checked first.
    priority: 1
    scaling_components:
      - component_id: k8s_corddns
        # replicas must string type
        replicas: "2"
`;
