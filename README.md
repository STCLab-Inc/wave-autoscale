![drawing](https://docs.google.com/presentation/d/1mE9LXq-Z780BNVzUUA4eizlbridFnDp11PQD6tCqapw/export/png)

# Wave Autoscale
**Proactive and Reactive Event Driven Autoscaling for EC2, Kubernetes, Serverless, and more.**

Developed by [STCLAB](https://www.stclab.com)

**Table of contents**
- [What is Wave Autoscale?](#what-is-wave-autoscale)
  - [Principles](#principles)
  - [Use Cases](#use-cases)
- [Getting started](#getting-started)
- [Configuration](#configuration)
  - [Metrics](#metrics)
  - [Scaling Components](#scaling-components)
  - [Scaling Plans](#scaling-plans)
- [Development](#development)
  - [Preparation](#preparation)
- [Additional Notes](#additional-notes)
- [Contributing](#contributing)
- [License](#license)
- [Support](#support)
- [Acknowledgments](#acknowledgments)
  - [Principles](#principles-1)
  - [Use Cases](#use-cases-1)
- [Getting started](#getting-started-1)
- [Configuration](#configuration-1)
  - [Metrics](#metrics-1)
  - [Scaling Components](#scaling-components-1)
  - [Scaling Plans](#scaling-plans-1)
- [Development](#development-1)
  - [Preparation](#preparation-1)
- [Additional Notes](#additional-notes-1)
- [Contributing](#contributing-1)
- [License](#license-1)
- [Support](#support-1)
- [Acknowledgments](#acknowledgments-1)

## What is Wave Autoscale?
Wave Autoscale is an advanced autoscaling service that provides both proactive and reactive event-driven autoscaling for a variety of platforms, including Amazon EC2, Kubernetes, serverless services Like AWS Lambda, and more. Designed as a coordinated solution to existing autoscaling technologies such as [AWS EC2 Autoscaling](https://aws.amazon.com/ec2/autoscaling/), [Kubernetes Karpenter](https://karpenter.sh/), and [Kubernetes Horizontal Pod Autoscaler (HPA)](https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/), Wave Autoscale extends their capabilities by incorporating both proactive and reactive scaling strategies. This ensures optimal resource allocation, improves application performance, and minimizes operational costs, resulting in a seamless end-user experience across diverse cloud environments.

Proactive autoscaling utilizes historical data and predictive analytics to anticipate demand fluctuations and adjust resources accordingly, while reactive autoscaling responds to real-time metrics and events to make immediate scaling decisions. Together, these approaches enable Wave Autoscale to deliver unparalleled efficiency and adaptability for your cloud applications.

Wave Autoscale can integrate with a variety of metrics from multiple sources, including Application Performance Monitoring (APM) tools such as [Datadog](https://www.datadoghq.com/), [New Relic](https://newrelic.com/), and [Dynatrace](https://www.dynatrace.com/), End-User Monitoring (EUM) solutions like [AppDynamics](https://www.appdynamics.com/), [Splunk](https://www.splunk.com/), and [Elastic APM](https://www.elastic.co/apm), [Prometheus](https://prometheus.io/) for monitoring and alerting, as well as system metrics like message counts in [Kafka](https://kafka.apache.org/) and [AWS SQS](https://aws.amazon.com/sqs/). This integration allows you to make more informed scaling decisions based on a comprehensive understanding of your application's performance and user experience.

### Principles

- **Reliability**: Built with reliability in mind, using [Rust](https://www.rust-lang.org/) to ensure optimal resource allocation and application performance.
- **Integration**: Seamlessly integrates with a wide range of metrics sources, including APM tools and system metrics, for easy incorporation into existing development workflows.
- **Customization**: Offers extensive customization options for SREs and cloud engineers to tailor the solution to their specific needs, beyond traditional autoscaling for services such as EC2.


### Use Cases

- **Traffic-aware allocation** for marketing campaigns with high traffic volume
- **Behavior-based tuning** for web and mobile applications based on SLOs and EUM metrics
- **Time-based scaling** for internal systems based on time and demand
- **Capacity scaling** for media streaming services during peak usage times
- **On-demand provisioning for serverless functions** based on workload demands
- **Volume-based allocation for IoT applications** based on data volume and processing requirements
- **Workload-based allocation** for batch processing jobs based on job size and complexity.
- **Data-driven allocation** for machine learning workloads based on data input size, model complexity, and processing requirements.
- **Player-focused allocation** for game servers based on player activity and demand

## Getting started

(TBD)

## Configuration

The configuration file consists of three main sections: Metrics, Scaling Components, and Scaling Plans. Below is an explanation of each section:

### Metrics

**Monitoring**
- [x] Prometheus
- [x] Amazon CloudWatch
- [ ] DataDog
- [ ] New Relic
- [ ] Dynatrace
- [ ] AppDynamics
- [ ] Splunk
- [ ] Elastic APM

**Messaging System**
- [ ] Kafka
- [ ] AWS SQS

**Workflow**
- [ ] Airflow
- [ ] Luigi
- [ ] MLFlow

**Data Analytics**
- [ ] Apache Spark
- [ ] Databricks

Metrics are used to define the data sources for the scaling plans. A metric has the following attributes:

- `kind`: Metric
- `id`: A unique identifier for the metric.
- `metric_kind`: The metric system being used (e.g., `prometheus`).
- `metadata`: A set of key-value pairs containing additional information about the metric.
- `polling_interval`: The interval (in milliseconds) at which the metric is polled.

Example:

```yaml
kind: Metric
id: prometheus_api_server_cpu_average_total
metric_kind: prometheus
metadata:
  endpoint: http://3.34.170.13:9090
  query: avg(100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[20s])) by (hostname) * 100))
polling_interval: 1000
```

### Scaling Components

- [x] Kubernetes Deployment Scaling
- [x] Amazon EC2 Autoscaling
- [ ] Amazon EMR managed scaling
- [ ] Amazon ECS Autoscaling
- [ ] Amazon Karpaenter
- [ ] AWS Lambda Provisioned Concurrency
- [ ] Amazon RDS Autoscaling
- [ ] Amazon DynamoDB Autoscaling
- [ ] Amazon Managed Streaming for Apache Kafka(MSK) Autoscaling
- [ ] Amazon SQS Autoscaling
- [ ] Amazon ElastiCache Autoscaling
- [ ] Amazon Neptune Autoscaling
- [ ] Amazon DocumentDB Autoscaling
- [ ] Amazon Elasticache Redis Cluster Autoscaling
- [ ] Amazon Elasticache Memcached Cluster Autoscaling
- [ ] Amazon SageMaker Endpoint Autoscaling

Scaling components define the resources that will be scaled. A scaling component has the following attributes:

- `kind`: ScalingComponent
- `id`: A unique identifier for the scaling component.
- `component_kind`: The platform or technology being used for scaling (e.g., `aws-ec2-autoscaling`).
- `metadata`: A set of key-value pairs containing additional information about the scaling component.

Example:

```yaml
kind: ScalingComponent
id: ec2_autoscaling_api_server
component_kind: aws-ec2-autoscaling
metadata:
  region: ap-northeast-2
  access_key: YOUR_AWS_ACCESS_KEY
  secret_key: YOUR_AWS_SECRET_KEY
  asg_name: tf-wa-20230322020900305100000006

```

### Scaling Plans

Scaling plans define the rules for scaling based on the metrics and scaling components. A scaling plan has the following attributes:

- `kind`: ScalingPlan
- `id`: A unique identifier for the scaling plan.
- `plans`: A list of individual scaling plan objects.

Each scaling plan object contains:

- `id`: A unique identifier for the individual scaling plan.
- `expression`: A JavaScript expression that returns a boolean value based on metric data.
- `priority`: The priority of the plan. Higher priority values will be checked first.
- `scaling_components`: A list of scaling component objects with their scaling configuration.

Example:

```yaml
kind: ScalingPlan
id: scaling_plan_ec2_autoscaling
plans:
  - id: scale-out-plan-1
    expression: "prometheus_api_server_cpu_average_total >= 30"
    priority: 1
    scaling_components:
      - component_id: ec2_autoscaling_api_server
        desired: "Math.floor(prometheus_api_server_cpu_average_total / 10)"

```


## Development


### Preparation
- Install [cargo-make](https://github.com/sagiegurari/cargo-make)
- Install [cargo-watch](https://crates.io/crates/cargo-watch)



**cargo-make**
```bash
cargo make test-controller-planner
```

**cargo-watch**
To watch the wave-autoscale package
```bash
cargo watch -x 'run -p wave-autoscale'
```

## Additional Notes

- Make sure to configure the correct access keys, secret keys, and other sensitive information for your scaling components.
- Test your expressions and scaling rules thoroughly to ensure they produce the desired outcome.
- Monitor the logs and output of your application to identify any issues or unexpected behavior.

## Contributing

If you'd like to contribute to this project, please follow these steps:

1. Fork the repository.
2. Create a branch for your feature or bugfix.
3. Implement and test your changes.
4. Create a pull request for review.

## License

This project is licensed under the [MIT License](https://chat.openai.com/LICENSE).

## Support

If you encounter any issues or need help with the project, please open an issue on the project's GitHub page, and we'll be happy to assist you.

## Acknowledgments

We would like to thank all the contributors and users of this project for their support and valuable feedback. Your contributions help make this project better for everyone.
nts.

Proactive autoscaling utilizes historical data and predictive analytics to anticipate demand fluctuations and adjust resources accordingly, while reactive autoscaling responds to real-time metrics and events to make immediate scaling decisions. Together, these approaches enable Wave Autoscale to deliver unparalleled efficiency and adaptability for your cloud applications.

Wave Autoscale can integrate with a variety of metrics from multiple sources, including Application Performance Monitoring (APM) tools such as [Datadog](https://www.datadoghq.com/), [New Relic](https://newrelic.com/), and [Dynatrace](https://www.dynatrace.com/), End-User Monitoring (EUM) solutions like [AppDynamics](https://www.appdynamics.com/), [Splunk](https://www.splunk.com/), and [Elastic APM](https://www.elastic.co/apm), [Prometheus](https://prometheus.io/) for monitoring and alerting, as well as system metrics like message counts in [Kafka](https://kafka.apache.org/) and [AWS SQS](https://aws.amazon.com/sqs/). This integration allows you to make more informed scaling decisions based on a comprehensive understanding of your application's performance and user experience.

### Principles

- Reliability: Built with reliability in mind, using [Rust](https://www.rust-lang.org/) to ensure optimal resource allocation and application performance.
- Integration: Seamlessly integrates with a wide range of metrics sources, including APM tools and system metrics, for easy incorporation into existing development workflows.
- Customization: Offers extensive customization options for SREs and cloud engineers to tailor the solution to their specific needs, beyond traditional autoscaling for services such as EC2.


### Use Cases

- **Traffic-aware allocation** for marketing campaigns with high traffic volume
- **Behavior-based tuning** for web and mobile applications based on SLOs and EUM metrics
- **Time-based scaling** for internal systems based on time and demand
- **Capacity scaling** for media streaming services during peak usage times
- **On-demand provisioning for serverless functions** based on workload demands
- **Volume-based allocation for IoT applications** based on data volume and processing requirements
- **Workload-based allocation** for batch processing jobs based on job size and complexity.
- **Data-driven allocation** for machine learning workloads based on data input size, model complexity, and processing requirements.
- **Player-focused allocation** for game servers based on player activity and demand

## Getting started

(TBD)

## Configuration

The configuration file consists of three main sections: Metrics, Scaling Components, and Scaling Plans. Below is an explanation of each section:

### Metrics

**Monitoring**
- [x] Prometheus
- [ ] Amazon CloudWatch
- [ ] DataDog
- [ ] New Relic
- [ ] Dynatrace
- [ ] AppDynamics
- [ ] Splunk
- [ ] Elastic APM

**System Metrics**
- [ ] Kafka
- [ ] AWS SQS


Metrics are used to define the data sources for the scaling plans. A metric has the following attributes:

- `kind`: Metric
- `id`: A unique identifier for the metric.
- `metric_kind`: The metric system being used (e.g., `prometheus`).
- `metadata`: A set of key-value pairs containing additional information about the metric.
- `polling_interval`: The interval (in milliseconds) at which the metric is polled.

Example:

```yaml
kind: Metric
id: prometheus_api_server_cpu_average_total
metric_kind: prometheus
metadata:
  endpoint: http://3.34.170.13:9090
  query: avg(100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[20s])) by (hostname) * 100))
polling_interval: 1000
```

### Scaling Components


- [x] Amazon EC2 Autoscaling
- [ ] Amazon ECS Autoscaling
- [ ] Amazon EKS Cluster Autoscaling
- [ ] Amazon Karpenter
- [ ] AWS Lambda Provisioned Concurrency
- [ ] Amazon RDS Autoscaling
- [ ] Amazon DynamoDB Autoscaling
- [ ] Amazon Managed Streaming for Apache Kafka(MSK) Autoscaling
- [ ] Amazon SQS Autoscaling
- [ ] Amazon ElastiCache Autoscaling
- [ ] Amazon Neptune Autoscaling
- [ ] Amazon DocumentDB Autoscaling
- [ ] Amazon Elasticache Redis Cluster Autoscaling
- [ ] Amazon Elasticache Memcached Cluster Autoscaling
- [ ] Amazon SageMaker Endpoint Autoscaling

Scaling components define the resources that will be scaled. A scaling component has the following attributes:

- `kind`: ScalingComponent
- `id`: A unique identifier for the scaling component.
- `component_kind`: The platform or technology being used for scaling (e.g., `aws-ec2-autoscaling`).
- `metadata`: A set of key-value pairs containing additional information about the scaling component.

Example:

```yaml
kind: ScalingComponent
id: ec2_autoscaling_api_server
component_kind: aws-ec2-autoscaling
metadata:
  region: ap-northeast-2
  access_key: YOUR_AWS_ACCESS_KEY
  secret_key: YOUR_AWS_SECRET_KEY
  asg_name: tf-wa-20230322020900305100000006

```

### Scaling Plans

Scaling plans define the rules for scaling based on the metrics and scaling components. A scaling plan has the following attributes:

- `kind`: ScalingPlan
- `id`: A unique identifier for the scaling plan.
- `plans`: A list of individual scaling plan objects.

Each scaling plan object contains:

- `id`: A unique identifier for the individual scaling plan.
- `expression`: A JavaScript expression that returns a boolean value based on metric data.
- `priority`: The priority of the plan. Higher priority values will be checked first.
- `scaling_components`: A list of scaling component objects with their scaling configuration.

Example:

```yaml
kind: ScalingPlan
id: scaling_plan_ec2_autoscaling
plans:
  - id: scale-out-plan-1
    expression: "prometheus_api_server_cpu_average_total >= 30"
    priority: 1
    scaling_components:
      - component_id: ec2_autoscaling_api_server
        desired: "Math.floor(prometheus_api_server_cpu_average_total / 10)"

```


## Development


### Preparation
- Install [cargo-make](https://github.com/sagiegurari/cargo-make)
- Install [cargo-watch](https://crates.io/crates/cargo-watch)



**cargo-make**
```bash
cargo make test-controller-planner
```

**cargo-watch**
To watch the wave-autoscale package
```bash
cargo watch -x 'run -p wave-autoscale'
```

## Additional Notes

- Make sure to configure the correct access keys, secret keys, and other sensitive information for your scaling components.
- Test your expressions and scaling rules thoroughly to ensure they produce the desired outcome.
- Monitor the logs and output of your application to identify any issues or unexpected behavior.

## Contributing

If you'd like to contribute to this project, please follow these steps:

1. Fork the repository.
2. Create a branch for your feature or bugfix.
3. Implement and test your changes.
4. Create a pull request for review.

## License

This project is licensed under the [MIT License](https://chat.openai.com/LICENSE).

## Support

If you encounter any issues or need help with the project, please open an issue on the project's GitHub page, and we'll be happy to assist you.

## Acknowledgments

We would like to thank all the contributors and users of this project for their support and valuable feedback. Your contributions help make this project better for everyone.
