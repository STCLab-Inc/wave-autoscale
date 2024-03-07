# AWS - ECS

### Infra Structure Setup
1. Create Cluster
  - Cluster name: `wa-sample-ecs-cluster`
  - Infrastructure:  `AWS Fargate (serverless)`, `Amazon EC2 instances`
    - Create new ASG
    - Provisioning model: `On-Demand`
    - Desired capacity: minimum 1 / maximum 3
2. Create New task definition with JSON
  - executionRoleArn: update `ecsTaskExecutionRole` Arn
  - `ecsTaskExecutionRole` attached with `AmazonECSTaskExecutionRolePolicy`
```json
{
    "family": "wa-sample-ecs-task",
    "containerDefinitions": [
        {
            "name": "wa-sample-ecs-container",
            "image": "public.ecr.aws/kishorj/docker-2048:latest",
            "cpu": 1024,
            "memory": 2048,
            "portMappings": [
                {
                    "name": "port_80",
                    "containerPort": 80,
                    "hostPort": 80,
                    "protocol": "tcp",
                    "appProtocol": "http"
                }
            ],
            "essential": true,
            "environment": [],
            "environmentFiles": [],
            "mountPoints": [],
            "volumesFrom": [],
            "ulimits": [],
            "logConfiguration": {
                "logDriver": "awslogs",
                "options": {
                    "awslogs-create-group": "true",
                    "awslogs-group": "/ecs/wa-sample-ecs-task",
                    "awslogs-region": "ap-northeast-2",
                    "awslogs-stream-prefix": "ecs"
                },
                "secretOptions": []
            },
            "systemControls": []
        }
    ],
    "executionRoleArn": "arn:aws:iam::<AccountId>:role/ecsTaskExecutionRole",
    "networkMode": "awsvpc",
    "requiresCompatibilities": [
        "EC2",
        "FARGATE"
    ],
    "cpu": "1024",
    "memory": "2048",
    "runtimePlatform": {
        "cpuArchitecture": "X86_64",
        "operatingSystemFamily": "LINUX"
    }
}
```
3. Create Service
  - Compute options: Launch type `FARGATE`
  - Application type: `Service`
  - Task Definition Family: `wa-sample-ecs-task`
  - Service name: `wa-sample-ecs-service-fargate`
  - Service type: `Replica`
  - Desired tasks: 1


### Permissions
```yaml
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "Statement1",
            "Effect": "Allow",
            "Action": [
                "ecs:UpdateService"
            ],
            "Resource": [
                "*"
            ]
        }
    ]
}
```


### Sample Scaling Plan
examples/scaling-component/aws/aws_ecs_service_scaling.yaml


### Scaling Component
- **cluster_name**  
  *String | Required*
  - AWS ECS Cluster name
- **service_name**  
  *String | Required*
  - AWS ECS Service name
- **region**  
  *String | Required*
  - AWS ECS region


### Scaling Plan - component metadata
- **desired**  
  *Number | Required*
  - Change the Desired Capacity of the ECS Service.

```yaml
# example
scaling_components:
  - component_id: wa_sample_component_aws_ecs_service_scaling
    desired: 3
```

