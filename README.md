# **wave-autoscale**

![drawing](https://docs.google.com/presentation/d/1mE9LXq-Z780BNVzUUA4eizlbridFnDp11PQD6tCqapw/export/png)

## **Preparation**
- Install [cargo-make](https://github.com/sagiegurari/cargo-make)
- Install [cargo-watch](https://crates.io/crates/cargo-watch)

## **How to Use**

### **cargo-make**
```bash
cargo make test-controller-planner
```

### **cargo-watch**
To watch the wave-autoscale package
```bash
cargo watch -x 'run -p wave-autoscale'
```


## **Configuration**

The configuration file consists of three main sections: Metrics, Scaling Components, and Scaling Plans. Below is an explanation of each section:

### **Metrics**

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

### **Scaling Components**

Scaling components define the resources that will be scaled. A scaling component has the following attributes:

- **`kind`**: ScalingComponent
- **`id`**: A unique identifier for the scaling component.
- **`component_kind`**: The platform or technology being used for scaling (e.g., **`aws-ec2-autoscaling`**).
- **`metadata`**: A set of key-value pairs containing additional information about the scaling component.

Example:

```
yamlCopy code
kind: ScalingComponent
id: ec2_autoscaling_api_server
component_kind: aws-ec2-autoscaling
metadata:
  region: ap-northeast-2
  access_key: YOUR_AWS_ACCESS_KEY
  secret_key: YOUR_AWS_SECRET_KEY
  asg_name: tf-wa-20230322020900305100000006

```

### **Scaling Plans**

Scaling plans define the rules for scaling based on the metrics and scaling components. A scaling plan has the following attributes:

- **`kind`**: ScalingPlan
- **`id`**: A unique identifier for the scaling plan.
- **`plans`**: A list of individual scaling plan objects.

Each scaling plan object contains:

- **`id`**: A unique identifier for the individual scaling plan.
- **`expression`**: A JavaScript expression that returns a boolean value based on metric data.
- **`priority`**: The priority of the plan. Higher priority values will be checked first.
- **`scaling_components`**: A list of scaling component objects with their scaling configuration.

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

## **Usage**
(TBD)

## **Additional Notes**

- Make sure to configure the correct access keys, secret keys, and other sensitive information for your scaling components.
- Test your expressions and scaling rules thoroughly to ensure they produce the desired outcome.
- Monitor the logs and output of your application to identify any issues or unexpected behavior.

## **Contributing**

If you'd like to contribute to this project, please follow these steps:

1. Fork the repository.
2. Create a branch for your feature or bugfix.
3. Implement and test your changes.
4. Create a pull request for review.

## **License**

This project is licensed under the **[MIT License](https://chat.openai.com/LICENSE)**.

## **Support**

If you encounter any issues or need help with the project, please open an issue on the project's GitHub page, and we'll be happy to assist you.

## **Acknowledgments**

We would like to thank all the contributors and users of this project for their support and valuable feedback. Your contributions help make this project better for everyone.