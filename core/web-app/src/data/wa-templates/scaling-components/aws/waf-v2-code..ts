export const WAF_V2_CODE = `kind: ScalingComponent
id: scaling_component_aws_waf_rate_limit
component_kind: aws-waf-v2
metadata:
  region: "{{ region }}"
  web_acl_id: "{{ web_acl_id }}"
  web_acl_name: "{{ web_acl_name }}"
  scope: "{{ scope }}"
---
kind: Metric
id: telegraf_cloudwatch_aws_waf_metrics
collector: telegraf
metadata:
  inputs:
    cloudwatch:
      - region: "{{ region }}"
        profile: "{{ profile }}"
        period: 1m
        delay: 1m
        interval: 1m
        namespaces: [AWS/WAFV2]
        metrics: # SELECT SUM(BlockedRequests) FROM SCHEMA("AWS/WAFV2", Region,Rule,WebACL) WHERE WebACL = '"{{ web_acl_name }}"' AND Region = '"{{ region }}"' AND Rule = '"{{ rule_name }}"'
          - names: [BlockedRequests] # This metric monitors the number of blocked web requests. In terms of scaling, a decrease in the "Blocked Requests" metric suggests a decline in blocked traffic, prompting consideration for scale-in. Conversely, an increase in this metric implies a rise in blocked traffic, making scale-out a relevant consideration.
            statistic_include: [sum] # Collect the metrics with the Sum statistic.
            dimensions:
              - name: Region
                value: "{{ region }}"
              - name: Rule
                value: "{{ rule_name }}"
              - name: WebACL
                value: "{{ web_acl_name }}"
  outputs:
    wave-autoscale: {}
---
kind: ScalingPlan
id: scaling_plan_aws_waf_rate_limit
metadata:
  title: scaling plan for aws waf rate limit
  cool_down: 0 # seconds
  interval: 60000 # milliseconds
plans:
  # When the number of blocked request is low (less than 100), the rate limit decreases (scaling in), which reflects the low usage.
  # When the number of blocked request is high (exceeding 1000), the rate limit (scaling out) to meet the high demand.
  - id: scaling_plan_aws_waf_rate_limit_scale_in
    description: scale-in rate limit
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_cloudwatch_aws_waf_metrics',
        name: 'cloudwatch_aws_waf_blocked_requests_sum',
        tags: {
          'region': '{{ region }}'
          'rule': '{{ rule_name }}'
          'web_acl': '{{ web_acl_name }}'
        },
        period_sec: 60
      }) < 100
    # Higher priority values will be checked first.
    priority: 1
    scaling_components:
      - component_id: scaling_component_aws_waf_rate_limit
        rule_name: "{{ rule_name }}"
        rate_limit: 100
  - id: scaling_plan_aws_waf_rate_limit_scale_out
    description: scale-out rate limit
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_cloudwatch_aws_waf_metrics',
        name: 'cloudwatch_aws_waf_blocked_requests_sum',
        tags: {
          'region': '{{ region }}'
          'rule': '{{ rule_name }}'
          'web_acl': '{{ web_acl_name }}'
        },
        period_sec: 60
      }) > 1000
    # Higher priority values will be checked first.
    priority: 2
    scaling_components:
      - component_id: scaling_component_aws_waf_rate_limit
        rule_name: "{{ rule_name }}"
        rate_limit: 1000
`;
