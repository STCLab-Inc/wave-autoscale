export const EC2_AUTOSCALING_CODE = `
kind: ScalingComponent
id: ec2_autoscaling
component_kind: aws-ec2-autoscaling
metadata:
  region: ap-northeast-1
  asg_name: ec2-asg-name-custom
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
id: aws_ec2_autoscaling_capacity
title: scaling plan for aws ec2 autoscaling capacity
  cool_down: 0 # seconds
  interval: 60000 # milliseconds
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
      - component_id: ec2_autoscaling
        desired: 10
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
      - component_id: ec2_autoscaling
        desired: 1`;