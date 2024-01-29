<img src="https://wave-autoscale-marketplace-resources.s3.amazonaws.com/wa-marketplace-logo.png" alt="drawing" style="width:400px;"/>

Wave Autoscale is an unified autoscaling & traffic management tool for clouds and Kubernetes. Customize autoscaling and traffic control of clouds to meet your availability and cost-efficiency needs.

![GitHub release (latest by date)](https://img.shields.io/github/v/release/STCLab-Inc/wave-autoscale)
![GitHub](https://img.shields.io/github/license/STCLab-Inc/wave-autoscale)
![GitHub contributors](https://img.shields.io/github/contributors/STCLab-Inc/wave-autoscale)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](code_of_conduct.md)


- Website: https://waveautoscale.com
- Documentation: [https://waveautoscale.com/docs](https://www.waveautoscale.com/docs)

## Get Started ##
There are several ways to install Wave Autoscale.

- [Install on Kubernetes using Helm](https://www.waveautoscale.com/docs/guide/getting-started/k8s-helm/)
- [Install on Kubernetes using Manifest](https://www.waveautoscale.com/docs/guide/getting-started/k8s-manifest/)
- [Install using Docker Compose](https://www.waveautoscale.com/docs/guide/getting-started/docker-compose/)
- [Install using Docker](https://www.waveautoscale.com/docs/guide/getting-started/docker/)


## Why Autoscaling Management?

Common usage pattern of autoscaling is to scale in and out based on CPU utilization configured in K8s manifests or AWS Console. However, the engineers cannot imagine that how many Pods or VMs are scaled in and out with the CPU utilization. It is also difficult to predict the cost of autoscaling. Like IaC (Infrastructure as Code), the engineers need to manage the autoscaling as code as a declarative way. Wave Autoscale suggests the declarative way called "Scaling Plan as Code" to manage the autoscaling. The engineers can manage autoscaling configurations as code to understand the autoscaling behavior and predict the cost of autoscaling.

- **Declarative Autoscaling**: Wave Autoscale allows you to define autoscaling rules and resource scales declaratively using YAML, providing a more predictable approach to your budget and resource allocation.
- **Beyond CPU Utilization Scaling**: Traditional autoscaling often relies on simple metrics like CPU utilization. While it's an essential factor, it doesn't always provide a complete picture. Wave Autoscale considers a wider range of metrics, such as the size of the message queue, the status of the database, custom business KPIs, and more, allowing for a more nuanced and responsive scaling strategy.
- **Simultaneous Scaling of Various Components**: In modern architectures, scaling just one component like Kubernetes pods may not suffice. Related components such as serverless functions, databases, caching layers, and more must be scaled concurrently. Wave Autoscale provides simultaneous multi-scaling capabilities that cover a wide array of components, ensuring that all interconnected parts of an application are scaled in harmony, thus preventing bottlenecks.
- **Cost-Effective Scaling**: Unnecessary over-provisioning leads to increased costs, while under-provisioning can impact performance. Wave Autoscale continuously monitors and adjusts resources based on actual usage, ensuring cost-efficient scaling without compromising on performance.
- **Business Event-Driven Scaling**: Traditional autoscaling is often based on technical metrics like CPU utilization, which may not always align with business needs. Wave Autoscale allows you to define custom scaling plans based on business events, such as marketing campaigns, product launches, and more, ensuring that resources are allocated based on business priorities.

Wave Autoscale is designed to overcome these challenges, providing a more tailored and responsive approach to autoscaling that aligns with both technical complexities and business needs.

## Use Cases ##
<img src="https://wave-autoscale-marketplace-resources.s3.amazonaws.com/app-icons/kubernetes.svg" alt="drawing" style="width:50px;"/>

**Kubernetes**
- **Scaling Replicas**: Scaling a number of replicas of a deployment with metrics (not by HPA)
- **CoreDNS Autoscaling**: Scaling CoreDNS pods based on metrics
- **Pod Right Sizing**: Dynamically right-sizing pods based on metrics
- **Istio Traffic Delay**: When a service or a database is heavyly loaded, delay can be used to protect it from overload
- **Istio Rate Limits**: When a service or a database is heavyly loaded, rate limiting can be used to protect it from overload

<img src="https://wave-autoscale-marketplace-resources.s3.amazonaws.com/app-icons/aws.svg" alt="drawing" style="width:50px;"/>

**AWS**
- **EC2 Auto Scaling**: Scaling EC2 instances based on metrics
- **ECS Auto Scaling**: Scaling ECS tasks based on metrics
- **Lambda Concurrency**: Adjust reserved concurrency and provisioned concurrency dynamically to archive optimal performance and efficiency
- **EMR on EC2**: Configure the on-demand or spot instances, step concurrency, and more
- **AWS WAF v2**: Adjust the rate limit of a WAF rule dynamically based on metrics

And there are more use cases regarding **Google Cloud**, **Azure**, **Cloudflare**, and more.

## Similar Projects ###
There are several notable projects offering unique capabilities. Here's a look at some of them:

- KEDA (Kubernetes Event-Driven Autoscaling): KEDA is a prominent open-source project that focuses on event-driven autoscaling for Kubernetes workloads. It supports a wide variety of event sources and is designed to seamlessly work with Kubernetes deployments. [Learn more about KEDA.](https://keda.sh/)

## Founded by ##
[<img src="https://github.com/STCLab-Inc/wave-autoscale/assets/114452/2dd6b2b4-1a96-4684-aa74-a2dd8bf70bad" width=100 />](https://cloud.stclab.com)

Wave Autoscale is founded by [STCLab Inc](https://cloud.stclab.com), a startup on a mission to provide reliable web traffic and resource management by allowing our clients to have control over infrastructure/server costs, reduce downtime, and further add value in enhancing their experience.

The “STC” is an abbreviation for “Surfing The Wave of Change” meaning only those who are prepared for the Wave of Change can surf and enjoy.

## License
Wave Autoscale is licensed under the [Apache License 2.0](https://github.com/STCLab-Inc/wave-autoscale/blob/main/LICENSE)