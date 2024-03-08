# AWS - WAF

### Infra Structure Setup
1. Create web ACL
  - Resource type: Regional resources
  - name: `wa-sample-wafv2`
2. Add rules
  - Add my own rules and rule groups
  - Rule type: Rule builder
  - name: `wa-sample-wafv2-rule`
  - type: `Rate-based rule`
  - Rate limit: 100


### Permissions
```yaml
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "Statement1",
            "Effect": "Allow",
            "Action": [
                "wafv2:GetWebACL",
                "wafv2:UpdateWebACL"
            ],
            "Resource": [
                "*"
            ]
        }
    ]
}
```


### Sample Scaling Plan
examples/scaling-component/aws/aws_wafv2.yaml


### Scaling Component
- **web_acl_id**  
  *String | Required*
  - AWS WAF ACLs ID
- **web_acl_name**  
  *String | Required*
  - AWS WAF ACLs Name
- **scope**  
  *String | Required*
  - AWS ACL Resource type (`cloudfront` or `regional` )
- **region**  
  *String | Required*
  - AWS ACL region


### Scaling Plan - component metadata
- **rule_name**  
  *String | Required*
  - AWS WAF Rule Name
- **rate_limit**  
  *String | Required*
  - AWS WAF Rule Rate limit
  - Rate limit must be between 100 and 20,000,000.
  - The maximum number of requests to allow during the specified time window that satisfy your criteria.

```yaml
# example
scaling_components:
  - component_id: wa_sample_component_aws_wafv2
    rule_name: wa-sample-wafv2-rule
    rate_limit: 100
```
