## Wave Autoscale - Open-source Autoscaling Solution for Cloud Engineers

![banner](https://github.com/STCLab-Inc/wave-autoscale/assets/114452/3b403fa2-52ec-4f29-841f-e48815a0208b)


- Website: https://waveautoscale.com
- Documentation: https://waveautoscale.com/docs/guide/introduction/

Wave Autoscale is an autoscaling solution that simplifies and automates complex autoscaling configurations within the cloud, adapting to the changing traffic demands.
Though cloud service providers such as AWS, Google Cloud, and Azure offer some autoscaling features, engineers often struggle to handle the many challenges associated with dynamic scaling demands.

- **Beyond CPU Utilization Scaling**: Traditional autoscaling often relies on simple metrics like CPU utilization. While it's an essential factor, it doesn't always provide a complete picture. Wave Autoscale considers a wider range of metrics, such as the size of the message queue, the status of the database, custom business KPIs, and more, allowing for a more nuanced and responsive scaling strategy.
- **Simultaneous Scaling of Various Components**: In modern architectures, scaling just one component like Kubernetes pods may not suffice. Related components such as serverless functions, databases, caching layers, and more must be scaled concurrently. Wave Autoscale provides simultaneous multi-scaling capabilities that cover a wide array of components, ensuring that all interconnected parts of an application are scaled in harmony, thus preventing bottlenecks.
- **Multi-Cloud, Multi-Region, Multi-Tenant Scaling**: Modern applications are often deployed across multiple cloud providers, regions, and tenants. After controlling each autoscaling component separately, it can be challenging to ensure that all parts of the application are scaled in harmony. Wave Autoscale offers a unified interface that allows you to manage all autoscaling components from a single dashboard, providing a more holistic approach to autoscaling.
- **Cost-Effective Scaling**: Unnecessary over-provisioning leads to increased costs, while under-provisioning can impact performance. Wave Autoscale continuously monitors and adjusts resources based on actual usage, ensuring cost-efficient scaling without compromising on performance.
- **Intelligent Traffic-Aware Scaling**: Seasonal variations, marketing campaigns, or product launches can cause sudden spikes in user traffic. Wave Autoscale reacts to these changes, ensuring that resources are scaled up promptly to meet demand and provide a seamless user experience.
- **Customized Scaling for Machine Learning Workloads**: Machine learning workloads may require specific scaling strategies based on the data input size, model complexity, and processing requirements. Wave Autoscale offers data-driven allocation, allowing engineers to define custom scaling plans that align with the unique demands of ML applications.

Wave Autoscale is designed to overcome these challenges, providing a more tailored and responsive approach to autoscaling that aligns with both technical complexities and business needs.

## ⚙️ How Wave Autoscale Works ##
![how-wave-autoscale-works](https://github.com/STCLab-Inc/wave-autoscale/assets/114452/b4945330-32e0-4591-b810-56f0d6b5e894)

Wave Autoscale works by letting users write down what they need for scaling. This includes [**scaling plan definitions**](https://www.waveautoscale.com/docs/guide/concepts/scaling-plans) and [**scaling component definitions**](https://www.waveautoscale.com/docs/guide/concepts/scaling-components). Within the scaling plan definitions, there are three ways to trigger scaling components:

1. **Metric-based Events (Logical Expression in JavaScript)**: Users can write conditions using logical expressions in JavaScript that, when met, will initiate scaling.
   ```yaml
   // In a yaml file, if the number of connections to the Postgres exceeds 300
   expression: get({ metric_id: 'postgres', name: 'db_connections' }) > 300
   ```
2. **Time-based Events (Cron Expression)**: Users can schedule scaling actions at specific times using cron expressions.
   ```yaml
   // In a yaml file, every day at 1am
   cron_expression: "0 0 1 * * * *"
   ```
3. **Custom Events (Custom Trigger by HTTP)**: Users can create their own triggers using HTTP, allowing for unique and specialized scaling actions.

If a user wants to include measurements like traffic or usage, they can write down [**metric definitions**](https://www.waveautoscale.com/docs/guide/concepts/metrics). The system then looks at these instructions, especially the rules in the scaling plan definitions. When those rules are met, the scaling components are triggered. This means that the system can change and adjust automatically to different needs.

### Features

- **Unified Autoscaling Management**: Centralizes the management of various autoscaling components like Kubernetes (k8s), EC2 Autoscaling, and more in one place, providing a single dashboard for total control and transparency.
- **Wide Range of Integrations**: Connects with numerous metrics sources such as APM tools, system metrics, and monitoring solutions, offering a comprehensive understanding of application performance.
- **Simultaneous Multi-Scaling**: Wave Autoscale is a powerful tool that enables the control and management of various scaling components, including VMs (such as Amazon EC2 instances), container orchestration systems (like Kubernetes), and serverless platforms (such as provisioned concurrency). What sets Wave Autoscale apart is its ability to handle these different scaling components simultaneously, providing a comprehensive solution for managing scalability in a diverse infrastructure environment.
- **Flexible Scaling Policies**: Wave Autoscale allows you to define custom scaling policies based on your specific needs, such as traffic volume, resource utilization, or SLOs.

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



## 🔗 Integrations ##

### Scaling Components
|   |AWS|Google Cloud|Azure|
|---|---|---|---|
|Virtual Machine Group|✅ EC2 Autoscaling Group|✅ Managed Instance Group|✅ Azure Virtual Machine Scale Sets|
|Container Service|✅ Amazon ECS|✅ Cloud Run|🚧 Azure Container Apps|
|Kubernetes|✅ Amazon EKS (Deployment)|✅ Google Kubernetes Engine (Deployment)|✅ Managed Kubernetes Service (Deployment)|
|Serverless Function|✅ AWS Lambda (Reserved Concurrency, Provisioned Concurrency)|✅ Cloud Functions |✅ Azure Functions|
|Database|✅ DynamoDB|||
|Data Workloads|✅ Amazon EMR on EC2|🚧 Cloud Dataproc||
|In-Memory Database|🚧 Amazon ElastiCache|🚧 Memorystore|🚧 Cache for Redis|


### Metrics ###
For collecting metrics related to traffic or usage, Wave Autoscale leverages the capabilities of open-source tools Telegraf and Vector.

- [Telegraf](https://github.com/influxdata/telegraf): With support for about 250 input plugins, Telegraf offers an extensive range of metric collection possibilities. It's known for its adaptability to various environments and ease of configuration. [The MIT license](https://github.com/influxdata/telegraf/blob/master/LICENSE). [Learn more about Telegraf input plugins.](https://docs.influxdata.com/telegraf/v1.27/plugins/)
- [Vector](https://github.com/vectordotdev/vector): Supporting around 40 different sources, Vector provides a high-performance solution for collecting logs, metrics, and events. Its unified data model and strong reliability make it a valuable tool in monitoring and scaling. [The MPL 2.0 license](https://github.com/vectordotdev/vector/blob/master/LICENSE). [Learn more about Vector sources.](https://vector.dev/docs/reference/configuration/sources/)

These tools collectively empower Wave Autoscale to accurately monitor and adapt to the ever-changing demands of various cloud environments.


## 📄 Information ##

### Similar Projects ### 
There are several notable projects offering unique capabilities. Here's a look at some of them:

- KEDA (Kubernetes Event-Driven Autoscaling): KEDA is a prominent open-source project that focuses on event-driven autoscaling for Kubernetes workloads. It supports a wide variety of event sources and is designed to seamlessly work with Kubernetes deployments. [Learn more about KEDA.](https://keda.sh/)

### Founded by ###

[<img src="https://github.com/STCLab-Inc/wave-autoscale/assets/114452/2dd6b2b4-1a96-4684-aa74-a2dd8bf70bad" width=200 />](https://stclab.com)

[STCLab Homepage](https://surffy.io)

## License

```
Copyright 2023 STCLab, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

   https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

```

