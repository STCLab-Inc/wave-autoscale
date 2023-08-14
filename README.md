## Wave Autoscale - Optimize Cloud Autoscaling: Peak Performance, Cost-Efficient

![banner](https://github.com/STCLab-Inc/wave-autoscale/assets/114452/b9afc3c9-53a3-4cd2-9696-5b4387abfc73)

Wave Autoscale is an autoscaling solution that simplifies and automates complex autoscaling configurations within the cloud, adapting to the changing traffic demands.
Though cloud service providers such as AWS, Google Cloud, and Azure offer some autoscaling features, they often struggle to handle the many challenges associated with dynamic scaling demands.
- **Complexity of Managing Modern Architectures**: Modern architectures often consist of various interlinked components like microservices or serverless functions. Managing and scaling these intricate structures can become overwhelming and time-consuming.
- **Disconnect between Monitoring Data and Autoscaling Actions**: The information gained from monitoring data is invaluable, but there can often be a significant gap between this insight and the autoscaling actions that are undertaken. This disconnect might lead to less-than-optimal scaling decisions and hinder the application's ability to adapt efficiently to fluctuating demands.
- **Bridging Business Requirements and Autoscaling Options**: Each business has unique workload patterns and performance needs. Current autoscaling solutions may not offer enough flexibility or customization to meet these specific requirements, leading to challenges in aligning business goals with autoscaling strategies.

Wave Autoscale is designed to overcome these challenges, providing a more tailored and responsive approach to autoscaling that aligns with both technical complexities and business needs.

---

## ⚙️ How Wave Autoscale Works ##
Wave Autoscale works by letting users write down what they need for scaling. This includes **scaling plan definitions** and **scaling component definitions**. Within the scaling plan definitions, there are three ways to trigger scaling components:

1. **Metric-based Events (Logical Expression in JavaScript)**: Users can write conditions using logical expressions in JavaScript that, when met, will initiate scaling.
2. **Time-based Events (Cron Expression)**: Users can schedule scaling actions at specific times using cron expressions.
3. **Custom Events (Custom Trigger by HTTP)**: Users can create their own triggers using HTTP, allowing for unique and specialized scaling actions.

If a user wants to include measurements like traffic or usage, they can write down **metric definitions**. The system then looks at these instructions, especially the rules in the scaling plan definitions. When those rules are met, the scaling components are triggered. This means that the system can change and adjust automatically to different needs.


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


---

## 💻 Under The Hood ##

### Scaling Components
|   |AWS|Google Cloud|Azure|
|---|---|---|---|
|Virtual Machine Group|✅ EC2 Autoscaling Group|✅ Managed Instance Group|✅ Azure Virtual Machine Scale Sets|
|Container Service|✅ Amazon ECS|✅ Cloud Run|🚧 Azure Container Apps|
|Kubernetes|✅ Amazon EKS (Deployment)|✅ Google Kubernetes Engine (Deployment)|✅ Managed Kubernetes Service (Deployment)|
|Serverless Function|✅ AWS Lambda (Reserved Concurrency, Provisioned Concurrency)|✅ Cloud Functions |🚧 Azure Functions|
|Database|✅ DynamoDB|||
|Data Workloads|✅ Amazon EMR on EC2|🚧 Cloud Dataproc||
|In-Memory Database|🚧 Amazon ElastiCache|🚧 Memorystore|🚧 Cache for Redis|

### Metrics ###
For collecting metrics related to traffic or usage, Wave Autoscale leverages the capabilities of open-source tools Telegraf and Vector.

- Telegraf: With support for about 250 input plugins, Telegraf offers an extensive range of metric collection possibilities. It's known for its adaptability to various environments and ease of configuration. [Learn more about Telegraf input plugins.](https://docs.influxdata.com/telegraf/v1.27/plugins/)
- Vector: Supporting around 40 different sources, Vector provides a high-performance solution for collecting logs, metrics, and events. Its unified data model and strong reliability make it a valuable tool in monitoring and scaling. [Learn more about Vector sources.](https://vector.dev/docs/reference/configuration/sources/)

These tools collectively empower Wave Autoscale to accurately monitor and adapt to the ever-changing demands of various cloud environments.

---
## 📄 Resources ##

### Similar Projects ### 
There are several notable projects offering unique capabilities. Here's a look at some of them:

- KEDA (Kubernetes Event-Driven Autoscaling): KEDA is a prominent open-source project that focuses on event-driven autoscaling for Kubernetes workloads. It supports a wide variety of event sources and is designed to seamlessly work with Kubernetes deployments. [Learn more about KEDA.](https://keda.sh/)

---
© [STCLab, Inc](https://stclab.com/). All materials licensed under ...
