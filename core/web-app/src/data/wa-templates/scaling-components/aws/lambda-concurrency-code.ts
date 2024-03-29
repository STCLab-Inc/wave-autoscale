export const LAMBDA_CONCURRENCY_CODE = `kind: ScalingComponent
id: scaling_component_aws_lambda_function
component_kind: aws-lambda
metadata:
  region: "{{ region }}"
  function_name: "{{ function_name }}"
  qualifier: "{{ qualifier }}"
---
kind: Metric
id: telegraf_cloudwatch_aws_lambda_metrics
collector: telegraf
metadata:
  inputs:
    cloudwatch:
      - region: "{{ region }}"
        profile: "{{ profile }}"
        period: 1m
        delay: 1m
        interval: 1m
        namespaces: [AWS/Lambda]
        metrics: # SELECT AVG(Invocations) FROM SCHEMA("AWS/Lambda", FunctionName) WHERE FunctionName = '"{{ function_name }}"'
          - names: [Invocations] # Tracks the number of invocations of functions. This metric represents the workload of the function. An increase in invocations indicates an increase in the function's workload, potentially requiring scale-out. For example, if the Invocations metric exceeds a certain threshold, you can perform scale-out by adding additional replicas of the function.
            statistic_include: [sum] # Collect the metrics with the Sum statistic.
            dimensions:
              - name: FunctionName
                value: "{{ function_name }}"
      - region: "{{ region }}"
        profile: "{{ profile }}"
        period: 1m
        delay: 1m
        interval: 1m
        namespaces: [AWS/Lambda]
        metrics: # SELECT AVG(Duration) FROM SCHEMA("AWS/Lambda", FunctionName) WHERE FunctionName = '"{{ function_name }}"'
          - names: [Duration] # Tracks the execution time of functions. An increase in function execution time can impact scaling decisions. Functions with longer execution times may consume more resources, potentially necessitating scale-out. Additionally, functions with long execution times can also impact throttling and costs.
            statistic_include: [average] # Collect the metrics with the Average statistic, or Maximum statistic.
            dimensions:
              - name: FunctionName
                value: "{{ function_name }}"
      - region: "{{ region }}"
        profile: "{{ profile }}"
        period: 1m
        delay: 1m
        interval: 1m
        namespaces: [AWS/Lambda]
        metrics: # SELECT SUM(Throttles) FROM SCHEMA("AWS/Lambda", FunctionName) WHERE FunctionName = '"{{ function_name }}"'
          - names: [Throttles] # Tracks the number of times the function has been throttled. Throttling occurs when the concurrent execution limit of the function is exceeded. Throttles can indicate the need for scale-out or adjustments in resource allocation for the function.
            statistic_include: [sum] # Collect the metrics with the Sum statistic.
            dimensions:
              - name: FunctionName
                value: "{{ function_name }}"
  outputs:
    wave-autoscale: {}
---
kind: ScalingPlan
id: scaling_plan_aws_lambda_function
metadata:
  title: scaling plan for aws lambda function
  cool_down: 0 # seconds
  interval: 60000 # milliseconds
plans:
  # When the number of invocations is low (less than 30 or up to 60), the concurrency decreases (scaling in), which reflects the low usage.
  # When the number of invocations is high (exceeding 60 or surpassing 120), the concurrency increases (scaling out) to meet the high demand.
  - id: scaling_plan_aws_lambda_function_scale_in_1
    description: scale-in concurrency
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_cloudwatch_aws_lambda_metrics',
        name: 'cloudwatch_aws_lambda_invocations_sum',
        tags: {
          'function_name': '{{ function_name }}'
        },
        period_sec: 60
      }) < 30
    # Higher priority values will be checked first.
    priority: 2
    scaling_components:
      - component_id: scaling_component_aws_lambda_function
        provisioned_concurrency: 5
  - id: scaling_plan_aws_lambda_function_scale_in_2
    description: scale-in concurrency
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_cloudwatch_aws_lambda_metrics',
        name: 'cloudwatch_aws_lambda_invocations_sum',
        tags: {
          'function_name': '{{ function_name }}'
        },
        period_sec: 60
      }) <= 60
    # Higher priority values will be checked first.
    priority: 1
    scaling_components:
      - component_id: scaling_component_aws_lambda_function
        provisioned_concurrency: 10
  - id: scaling_plan_aws_lambda_function_scale_out_1
    description: scale-out concurrency
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_cloudwatch_aws_lambda_metrics',
        name: 'cloudwatch_aws_lambda_invocations_sum',
        tags: {
          'function_name': '{{ function_name }}'
        },
        period_sec: 60
      }) > 60
    # Higher priority values will be checked first.
    priority: 3
    scaling_components:
      - component_id: scaling_component_aws_lambda_function
        provisioned_concurrency: 20
  - id: scaling_plan_aws_lambda_function_scale_out_2
    description: scale-out concurrency
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_cloudwatch_aws_lambda_metrics',
        name: 'cloudwatch_aws_lambda_invocations_sum',
        tags: {
          'function_name': '{{ function_name }}'
        },
        period_sec: 60
      }) > 120
    # Higher priority values will be checked first.
    priority: 4
    scaling_components:
      - component_id: scaling_component_aws_lambda_function
        provisioned_concurrency: 40
  # When the average execution time is short (less than 0.25 seconds or up to 0.5 seconds), the system deems fewer resources as sufficient and reduces concurrency.
  # When the average execution time is long (exceeding 1 second or surpassing 2 seconds), the system determines that more resources are needed and increases concurrency to prevent performance degradation.
  - id: scaling_plan_aws_lambda_function_scale_in_3
    description: scale-in concurrency
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_cloudwatch_aws_lambda_metrics',
        name: 'cloudwatch_aws_lambda_duration_average',
        tags: {
          'function_name': '{{ function_name }}'
        },
        period_sec: 60
      }) < 0.25
    # Higher priority values will be checked first.
    priority: 6
    scaling_components:
      - component_id: scaling_component_aws_lambda_function
        provisioned_concurrency: 5
  - id: scaling_plan_aws_lambda_function_scale_in_4
    description: scale-in concurrency
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_cloudwatch_aws_lambda_metrics',
        name: 'cloudwatch_aws_lambda_duration_average',
        tags: {
          'function_name': '{{ function_name }}'
        },
        period_sec: 60
      }) <= 0.5
    # Higher priority values will be checked first.
    priority: 5
    scaling_components:
      - component_id: scaling_component_aws_lambda_function
        provisioned_concurrency: 10
  - id: scaling_plan_aws_lambda_function_scale_out_3
    description: scale-out concurrency
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_cloudwatch_aws_lambda_metrics',
        name: 'cloudwatch_aws_lambda_duration_average',
        tags: {
          'function_name': '{{ function_name }}'
        },
        period_sec: 60
      }) > 1
    # Higher priority values will be checked first.
    priority: 7
    scaling_components:
      - component_id: scaling_component_aws_lambda_function
        provisioned_concurrency: 20
  - id: scaling_plan_aws_lambda_function_scale_out_4
    description: scale-out concurrency
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_cloudwatch_aws_lambda_metrics',
        name: 'cloudwatch_aws_lambda_duration_average',
        tags: {
          'function_name': '{{ function_name }}'
        },
        period_sec: 60
      }) > 2
    # Higher priority values will be checked first.
    priority: 8
    scaling_components:
      - component_id: scaling_component_aws_lambda_function
        provisioned_concurrency: 40
  # When the throttling occurrences are few (less than 1 or up to 10), the system cuts back on excessive resources to optimize costs.
  # When throttling occurrences are numerous (exceeding 50 or surpassing 100), this indicates a lack of resources, and the system increases concurrency to address this issue.
  - id: scaling_plan_aws_lambda_function_scale_in_5
    description: scale-in concurrency
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_cloudwatch_aws_lambda_metrics',
        name: 'cloudwatch_aws_lambda_throttles_sum',
        tags: {
          'function_name': '{{ function_name }}'
        },
        period_sec: 60
      }) < 1
    # Higher priority values will be checked first.
    priority: 10
    scaling_components:
      - component_id: scaling_component_aws_lambda_function
        provisioned_concurrency: 5
  - id: scaling_plan_aws_lambda_function_scale_in_6
    description: scale-in concurrency
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_cloudwatch_aws_lambda_metrics',
        name: 'cloudwatch_aws_lambda_throttles_sum',
        tags: {
          'function_name': '{{ function_name }}'
        },
        period_sec: 60
      }) <= 10
    # Higher priority values will be checked first.
    priority: 9
    scaling_components:
      - component_id: scaling_component_aws_lambda_function
        provisioned_concurrency: 10
  - id: scaling_plan_aws_lambda_function_scale_out_5
    description: scale-out concurrency
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_cloudwatch_aws_lambda_metrics',
        name: 'cloudwatch_aws_lambda_throttles_sum',
        tags: {
          'function_name': '{{ function_name }}'
        },
        period_sec: 60
      }) > 50
    # Higher priority values will be checked first.
    priority: 11
    scaling_components:
      - component_id: scaling_component_aws_lambda_function
        provisioned_concurrency: 20
  - id: scaling_plan_aws_lambda_function_scale_out_6
    description: scale-out concurrency
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_cloudwatch_aws_lambda_metrics',
        name: 'cloudwatch_aws_lambda_throttles_sum',
        tags: {
          'function_name': '{{ function_name }}'
        },
        period_sec: 60
      }) > 100
    # Higher priority values will be checked first.
    priority: 12
    scaling_components:
      - component_id: scaling_component_aws_lambda_function
        provisioned_concurrency: 40`;
