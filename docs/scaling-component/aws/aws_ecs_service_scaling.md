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


### Scaling Plan - component metadata
- **desired**  
  *Number or String | Required*
  - Change the Desired Capacity of the Auto Scaling Group.
  - `desired` can be a number or an expression. The variables that can be used when using an expression are as follows.
    - `$desired`: Desired Capacity of the Auto Scaling Group
    - `$min`: Min Capacity of the Auto Scaling Group
    - `$max`: Max Capacity of the Auto Scaling Group
- **min**  
  *Number | Optional*
  - Change the Min Capacity of the Auto Scaling Group.
- **max**  
  *Number | Optional*
  - Change the Max Capacity of the Auto Scaling Group.

```yaml
# example 1
scaling_components:
  - component_id: wa_sample_component_aws_ec2_autoscaling
    desired: $desired + 1
```
```yaml
# example 2
scaling_components:
  - component_id: wa_sample_component_aws_ec2_autoscaling
    desired: 2
    min: 1
    max: 3
```

