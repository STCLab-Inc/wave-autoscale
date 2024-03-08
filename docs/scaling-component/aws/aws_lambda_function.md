# AWS - Lambda

### Infra Structure Setup
1. Create function
  - Function name: `wa-sample-lambda`
  - Use a blueprint: Hello world function
2. Version
  - Function > Versions > Publish new version > publish
  - Function > Version 1 > Configuration > Provisioned concurrency: `1`
3. Concurrency
  - Function > Configuration > Concurrency > edit
  - Use reserved concurrency: `1`


### Permissions
```yaml
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "Statement1",
            "Effect": "Allow",
            "Action": [
                "lambda:PutFunctionConcurrency",
                "lambda:PutProvisionedConcurrencyConfig"
            ],
            "Resource": [
                "*"
            ]
        }
    ]
}
```


### Sample Scaling Plan
examples/scaling-component/aws/aws_lambda.yaml


### Scaling Component
- **function_name**  
  *String | Required*
  - AWS Lambda Function name
- **qualifier**  
  *String | Optional (required when updating provisioned concurrency)*
  - AWS Lambda Provisioned concurrency configurations Qualifier name
- **region**  
  *String | Required*
  - AWS Lambda region


### Scaling Plan - component metadata
- **provisioned_concurrency**  
  *Number | Optional (required when updating provisioned concurrency)*
  - By using provisioned concurrency, pre-warmed Lambda function instances are kept in a ready state, allowing the function to execute immediately without any cold start delays when invoked.
  - Requested Provisioned Concurrency should not be greater than the reservedConcurrentExecution for function
- **reserved_concurrency**  
  *Number | Optional (required when updating reserved concurrency)*
  - Reserved Concurrency is a feature that sets the number of concurrent executions allocated for a specific Lambda function. This setting limits the maximum number of instances that the Lambda function can execute at the same time, thereby controlling the concurrency of the Lambda function.

```yaml
# example
scaling_components:
  - component_id: wa_sample_component_aws_lambda_function_provisioned
    provisioned_concurrency: 5
  - component_id: wa_sample_component_aws_lambda_function_reserved
    reserved_concurrency: 5
```

