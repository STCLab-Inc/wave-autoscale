# Kubernetes - json patch

### Infra Structure Setup
**Precondition**:  
Start with a pre-built *Kubernetes* & *Istio* environment.  

#### manifest install
apply the `wa-sample-k8s-json-patch.yaml` file to the Kubernetes cluster.
```bash
$ kubectl apply -f wa-sample-k8s-json-patch.yaml
```
```yaml
# wa-sample-k8s-json-patch.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: wa-sample-json-patch-nginx-dp
  labels:
    app: json-patch-nginx
spec:
  replicas: 1
  selector:
    matchLabels:
      app: json-patch-nginx
  template:
    metadata:
      labels:
        app: json-patch-nginx
    spec:
      containers:
      - name: nginx
        image: nginx:1.14.2
        ports:
        - containerPort: 80
---
apiVersion: v1
kind: Service
metadata:
  name: wa-sample-json-patch-sv
spec:
  type: NodePort
  selector:
    app: json-patch-nginx
  ports:
    - protocol: TCP
      port: 80
      targetPort: 80
---
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: wa-sample-json-patch-istio-vs
  namespace: istio-system
spec:
  hosts:
    - "*"
  gateways:
    - wa-sample-json-patch-istio-gateway
  http:
    - match:
        - uri:
            prefix: /
          port: 80
      route:
        - destination:
            host: wa-sample-json-patch-nginx-sv.default.svc.cluster.local
            port:
              number: 80
---
apiVersion: networking.istio.io/v1beta1
kind: Gateway
metadata:
  name: wa-sample-json-patch-istio-gateway
  namespace: istio-system
spec:
  selector:
    istio: ingressgateway
  servers:
    - port:
        number: 80
        name: wa-nginx
        protocol: HTTP
      hosts:
        - "*"
```

#### manifest uninstall
```bash
$ kubectl delete -f wa-sample-k8s-json-patch.yaml
```

### Sample Scaling Plan
- examples/scaling-component/kubernetes/istio_delay.yaml
- examples/scaling-component/kubernetes/istio_retry.yaml


### Scaling Plan - component metadata
- **namespace**  
  *String | Required*
  - Kubernetes namespace
- **name**  
  *String | Required*
  - Kubernetes resource name
- **api_version**  
  *String | Required*
  - Kubernetes resource api group and version
- **kind**  
  *String | Required*
  - Kubernetes resource kind
- **json_patch**  
  *Array(Map) | Required*
  - JSON Patch parameters in JSON format define the modification operation (`op`), target path (`path`), and when necessary, a new value (`value`) or additional path (`from`).
  - reference: [https://www.rfc-editor.org/rfc/rfc6902](https://www.rfc-editor.org/rfc/rfc6902)

```yaml
# example
scaling_components:
  - component_id: wa_sample_component_k8s_json_patch_istio_delay
    namespace: istio-system
    name: wa-sample-json-patch-istio-vs
    api_version: networking.istio.io/v1beta1
    kind: VirtualService
    json_patch:
      - op: add
        path: /spec/http/0/fault
        value: { "delay": { "fixedDelay": 10s } }
      - op: add
        path: /spec/http/0/fault/delay/percentage
        value: { "value": 100 }
```


### Permissions
Service Account (wave-autoscale-sa) should have edit permissions by deployment target.  
> Add ClusterRole, ClusterRoleBinding for wave-autoscale-sa.
