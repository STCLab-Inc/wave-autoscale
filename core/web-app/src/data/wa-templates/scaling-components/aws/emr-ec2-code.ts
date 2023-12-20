export const EMR_EC2_CODE = `kind: ScalingComponent
id: scaling_component_amazon_emr_ec2_instances
component_kind: amazon-emr-ec2
metadata:
  region: "{{ region }}"
  cluster_id: "{{ cluster_id }}"
  instance_group_id: "{{ instance_group_id }}"
---
kind: Metric
id: telegraf_cloudwatch_amazon_emr_ec2_metrics
collector: telegraf
metadata:
  inputs:
    cloudwatch:
      - region: "{{ region }}"
        profile: "{{ profile }}"
        period: 1m
        delay: 1m
        interval: 1m
        namespaces: [AWS/EC2]
        metrics: # SELECT MAX(CPUUtilization) FROM SCHEMA("AWS/EC2", AutoScalingGroupName) WHERE ClusterName = '"{{ cluster_name }}"'
          - names: [CPUUtilization] # CPU usage reflects the current workload of the instance. High CPU usage may indicate that the instance is overloaded with the current workload and can be a signal for scaling out. If CPU usage remains consistently high, it may be necessary to start additional instances to distribute the load.
            statistic_include: [maximum] # Collect the metrics with the Maximum statistic.
            dimensions:
              - name: AutoScalingGroupName
                value: "{{ auto_scaling_group_name }}"
  outputs:
    wave-autoscale: {}
---
kind: ScalingPlan
id: scaling_plan_amazon_emr_ec2_instances
title: scaling plan for amazon emr ec2 instances
  cool_down: 0 # seconds
  interval: 60000 # milliseconds
plans:
  # When the value of instance cpu utilization is low (less than 10), the instance configuration values decreases (scaling in), which reflects the low usage.
  # When the value of instance cpu utilization is high (exceeding 90), the instance configuration values increases (scaling out) to meet the high demand.
  - id: scaling_plan_amazon_emr_ec2_instances_scale_in
    description: scale-in instances
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_cloudwatch_amazon_emr_ec2_metrics',
        name: 'cloudwatch_aws_ec2_cpu_utilization_maximum',
        tags: {
          'instance_name': '{{ instance_name }}'
        },
        period_sec: 60
      }) < 10
    priority: 1
    scaling_components:
      - component_id: scaling_component_amazon_emr_ec2_instances
        instance_count: 2
        step_concurrency_level: 2
  - id: scaling_plan_amazon_emr_ec2_instances_scale_out
    description: scale-out instances
    # JavaScript expression that returns a boolean value.
    expression: >
      (get({
        metric_id: 'telegraf_cloudwatch_amazon_emr_ec2_metrics',
        name: 'cloudwatch_aws_ec2_cpu_utilization_maximum',
        tags: {
          'instance_name': '{{ instance_name }}'
        },
        period_sec: 60
      }) > 90
    priority: 2
    scaling_components:
      - component_id: scaling_component_amazon_emr_ec2_instances
        on_demand_capacity: 2
        spot_capacity: 2
        step_concurrency_level: 3
        on_demand_timeout_duration_minutes: 10
        spot_timeout_duration_minutes: 10
`;
