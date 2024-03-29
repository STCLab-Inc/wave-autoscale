export const ECS_AUTOASCALING_CODE = `kind: ScalingComponent
id: scaling_component_aws_ecs_service_scaling
component_kind: amazon-ecs
metadata:
  region: "{{ region }}"
  cluster_name: "{{ cluster_name }}"
  service_name: "{{ service_name }}"
---
kind: Metric
id: telegraf_cloudwatch_aws_ecs_service_scaling_metrics
collector: telegraf
metadata:
  inputs:
    cloudwatch:
      - region: "{{ region }}"
        profile: "{{ profile }}"
        period: 1m
        delay: 1m
        interval: 1m
        namespaces: [AWS/ECS]
        metrics: # SELECT MAX(CPUUtilization) FROM SCHEMA("AWS/ECS", ClusterName) WHERE ClusterName = '"{{ cluster_name }}"'
          - names: [CPUUtilization] # When CPUUtilization decreases, indicating a reduced need for CPU resources, it becomes a candidate for scale-in. This may involve terminating unnecessary EC2 instances or reducing Fargate tasks. Conversely, when CPUUtilization increases, indicating a higher demand for CPU resources, it becomes a candidate for scale-out. This may involve launching additional EC2 instances or expanding Fargate tasks.
            statistic_include: [maximum] # Collect the metrics with the Maximum statistic.
            dimensions:
              - name: ClusterName
                value: "{{ cluster_name }}"      
  outputs:
    wave-autoscale: {}
---
kind: ScalingPlan
id: scaling_plan_aws_ecs_service_scaling_instance
title: scaling plan for aws ecs service scaling instance
  cool_down: 0 # seconds
  interval: 60000 # milliseconds
plans:
  # When the value of instance cpu utilization is low (less than 10), the instance configuration values decreases (scaling in), which reflects the low usage.
  # When the value of instance cpu utilization is high (exceeding 90), the instance configuration values increases (scaling out) to meet the high demand.
  - id: scaling_plan_aws_ecs_service_scaling_instance_scale_in
    description: scale-in capacity
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_cloudwatch_aws_ecs_service_scaling_metrics',
        name: 'cloudwatch_aws_ec2_cpu_utilization_maximum',
        tags: {
          'instance_name': '{{ instance_name }}'
        },
        period_sec: 60
      }) < 10
    priority: 1
    scaling_components:
      - component_id: scaling_component_aws_ecs_service_scaling
        desired: 1
  - id: scaling_plan_aws_ecs_service_scaling_instance_scale_out
    description: scale-out capacity
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_cloudwatch_aws_ecs_service_scaling_metrics',
        name: 'cloudwatch_aws_ec2_cpu_utilization_maximum',
        tags: {
          'instance_name': '{{ instance_name }}'
        },
        period_sec: 60
      }) > 90
    priority: 2
    scaling_components:
      - component_id: scaling_component_aws_ecs_service_scaling
        desired: 10
`;
