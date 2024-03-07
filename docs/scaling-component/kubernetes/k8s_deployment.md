# Kubernetes - deployment

### Infra Structure Setup
**Precondition**:  
Start with a pre-built *Kubernetes* & *CoreDNS* environment.  


#### manifest install
apply the `wa-sample-k8s-deployment.yaml` file to the Kubernetes cluster.
```bash
$ kubectl apply -f wa-sample-k8s-deployment.yaml
```
```yaml
# wa-sample-k8s-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: wa-sample-deployment-nginx-dp
  labels:
    app: nginx
spec:
  replicas: 2
  selector:
    matchLabels:
      app: nginx
  template:
    metadata:
      labels:
        app: nginx
    spec:
      containers:
      - name: nginx
        image: nginx:1.14.2
        ports:
        - containerPort: 80
```

#### manifest uninstall
```bash
$ kubectl delete -f wa-sample-k8s-deployment.yaml
```


### Permissions
Service Account (wave-autoscale-sa) should have edit permissions by deployment target.  
> Add ClusterRole, ClusterRoleBinding for wave-autoscale-sa.


### Sample Scaling Plan
- examples/scaling-component/kubernetes/core_dns_replicas.yaml
- examples/scaling-component/kubernetes/deployment_replicas.yaml


### Scaling Component
- **namespace**  
  *String | Required*
  - Kubernetes namespace
- **name**  
  *String | Required*
  - Kubernetes deployment name
- **api_server_endpoint**  
  *String | Optional*
  - Kubernetes API server endpoint
- **ca_cert**  
  *String | Optional*
  - Kubernetes CA certificate


### Scaling Plan - component metadata
- **replicas**  
  *Number or String | Required*
  - Change the replicas of the deployment
  - `replicas` can be a number or an expression. The variables that can be used when using an expression are as follows.
    - `$replicas`: current number of replications for Deployment
    - `$unavailable_replicas`: the number of pod replicas that are not yet ready or available to serve traffic.
    - `$available_replicas`: the number of pod replicas that are ready and available to serve traffic.
    - `$ready_replicas`: the pod replicas that have passed their readiness checks and are considered ready to handle requests.
    - `$updated_replicas`: the number of pod replicas that have been successfully updated to the desired version or configuration specified in a deployment.

```yaml
# example
scaling_components:
  - component_id: wa_sample_component_k8s_deployment_core_dns_replicas
    replicas: $replicas + 1
```

