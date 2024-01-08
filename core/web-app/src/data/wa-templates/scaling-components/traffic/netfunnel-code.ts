export const NETFUNNEL_CODE = `kind: ScalingComponent
id: scaling_component_netfunnel_segment
component_kind: netfunnel
metadata:
  base_url: {{ base_url }}
  authorization: {{ authorization }}
  organization_id: "{{ organization_id }}"
  tenant_id: {{ tenant_id }}
  user_key: {{ user_key }}
  project_id: "{{ project_id }}"
  segment_id: "{{ segment_id }}"
---
kind: Metric
id: metric_netfunnel_segment
collector: vector
metadata:
  sources:
    my_source_id_1:
      type: http_client
      endpoint: {{ base_url }}/v2/wave/project/{{ project_id }}/segment-stats
      scrape_interval_secs: 5
      headers:
        Accept:
          - application/json, text/plain, */*
        Accept-Encoding:
          - gzip, deflate, br
        Accept-Language:
          - ko,en;q=0.9,en-US;q=0.8,ko-KR;q=0.7
        organizationId:
          - "{{ organization_id }}"
        tenantId:
          - {{ tenant_id }}
        userKey:
          - {{ user_key }}
        authorization:
          - {{ authorization }}
  transforms:
    my_transforms_id_1:
      inputs: [my_source_id_1]
      type: remap
      source: |-
        . = parse_json!(.message)
        tally = []
        for_each(array!(.data)) -> |_index, value| {
            tally = push(tally, {"name": "numbersWaiting", "tags": {"segmentId": to_string!(value.segmentId)}, "timestamp": value.timestamp, "gauge": {"value" : value.numbersWaiting}})
            tally = push(tally, {"name": "waitTime", "tags": {"segmentId": to_string!(value.segmentId)}, "timestamp": value.timestamp, "gauge": {"value" : value.waitTime}})
            tally = push(tally, {"name": "maxInflow", "tags": {"segmentId": to_string!(value.segmentId)}, "timestamp": value.timestamp, "gauge": {"value" : value.maxInflow}})
        }        
        tally
        . = tally
  sinks:
    my_sinks_id:
      type: wave-autoscale
      inputs: [my_transforms_id_1]
---
kind: ScalingPlan
id: scaling_plan_netfunnel_segment
metadata:
  title: Scaling Plan for NetFUNNEL Segment
  cool_down: 2 # seconds
  interval: 2000 # milliseconds
plans:
  - id: scale-out-plan-1
    description: Scale out if the number of waitings is greater than 500 when the max inflow is 1600.
    # JavaScript expression that returns a boolean value.
    expression: >
      get({
        metric_id: 'metric_netfunnel_segment',
        name: 'numbersWaiting',
        tags: {
          'segmentId': '{{ segment_id }}'
        },
        stats: 'max',
        period_sec: 1
      }) >= 500
      &&
      get({
        metric_id: 'metric_netfunnel_segment',
        name: 'maxInflow',
        tags: {
          'segmentId': '{{ segment_id }}'
        },
        stats: 'max',
        period_sec: 1
      }) == 1600
    # Higher priority values will be checked first.
    priority: 2
    scaling_components:
      - component_id: scaling_component_netfunnel_segment
        max_inflow: 3200
  - id: scale-out-plan-2
    description: Scale out if the number of waitings is greater than 1000 and if the wait time is greater than 50 when the max inflow is 40.
    # JavaScript expression that returns a boolean value.
    expression: >
      get({
        metric_id: 'metric_netfunnel_segment',
        name: 'numbersWaiting',
        tags: {
          'segmentId': '{{ segment_id }}'
        },
        stats: 'max',
        period_sec: 1
      }) >= 1000
      &&
      get({
        metric_id: 'metric_netfunnel_segment',
        name: 'maxInflow',
        tags: {
          'segmentId': '{{ segment_id }}'
        },
        stats: 'max',
        period_sec: 1
      }) == 40
    # Higher priority values will be checked first.
    priority: 1
    scaling_components:
      - component_id: scaling_component_netfunnel_segment
        max_inflow: 1600
  - id: scale-in-plan-1
    description: Scale in if the number of waitings is smaller than 1 and the max inflow is 1200.
    # JavaScript expression that returns a boolean value.
    expression: >
      get({
        metric_id: 'metric_netfunnel_segment',
        name: 'numbersWaiting',
        tags: {
          'segmentId': '{{ segment_id }}'
        },
        stats: 'max',
        period_sec: 1
      }) <= 1
      &&
      get({
        metric_id: 'metric_netfunnel_segment',
        name: 'maxInflow',
        tags: {
          'segmentId': '{{ segment_id }}'
        },
        stats: 'max',
        period_sec: 1
      }) == 1200
    # Higher priority values will be checked first.
    priority: 3
    scaling_components:
      - component_id: scaling_component_netfunnel_segment
        max_inflow: 60
  - id: scale-in-plan-2
    description: Scale out if the number of waitings is smaller than 1 and the max inflow is 3200.
    # JavaScript expression that returns a boolean value.
    expression: >
      get({
        metric_id: 'metric_netfunnel_segment',
        name: 'numbersWaiting',
        tags: {
          'segmentId': '{{ segment_id }}'
        },
        stats: 'max',
        period_sec: 1
      }) <= 1
      &&
      get({
        metric_id: 'metric_netfunnel_segment',
        name: 'maxInflow',
        tags: {
          'segmentId': '{{ segment_id }}'
        },
        stats: 'max',
        period_sec: 1
      }) == 3200
    # Higher priority values will be checked first.
    priority: 4
    scaling_components:
      - component_id: scaling_component_netfunnel_segment
        max_inflow: 1200

`;
