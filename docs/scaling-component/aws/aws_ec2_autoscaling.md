# AWS - EC2 Auto Scaling Groups

### Infra Structure Setup
1. Create launch template
    - Launch template name: `wa-sample-asg-launch-template`
2. Create Auto Scaling Group
    - Auto Scaling group name: `wa-sample-asg`
    - Launch template: `wa-sample-asg-launch-template`
    - choose Instance type: `t2.micro`
    - On-demand: `100%`
    - Desired capacity: `1`
    - Min capacity: `1`
    - Max capacity: `3`
    - Automatic scaling: `No scaling policies`


### Sample Scaling Plan
examples > scaling-component > aws > aws_ec2_autoscaling.yaml


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


### Permissions
```yaml
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "Statement1",
            "Effect": "Allow",
            "Action": [
                "autoscaling:UpdateAutoScalingGroup",
                "autoscaling:DescribeAutoScalingGroups"
            ],
            "Resource": [
                "*"
            ]
        }
    ]
}
```
