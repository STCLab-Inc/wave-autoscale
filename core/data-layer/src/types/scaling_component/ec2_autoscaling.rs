use ts_rs::TS;

#[derive(TS)]
#[ts(export, export_to = "../web-app/src/types/bindings/ec2-autoscaling.ts")]
pub struct EC2AutoscalingMetadata {
    pub region: String,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub asg_name: String,
}