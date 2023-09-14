use async_trait::async_trait;
use data_layer::ScalingComponentDefinition;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
pub mod amazon_dynamodb_table;
pub mod amazon_emr_ec2;
pub mod aws_ec2_autoscaling;
pub mod aws_ecs_service_scaling;
pub mod aws_lambda_function;
pub mod azure_functions_app;
pub mod azure_vmss_autoscaling;
pub mod gcp_mig_autoscaling;
pub mod google_cloud_functions_instance;
pub mod google_cloud_run_service;
pub mod k8s_deployment;
use self::{
    amazon_dynamodb_table::DynamoDbTableScalingComponent,
    amazon_emr_ec2::EMREC2AutoScalingComponent, aws_ec2_autoscaling::EC2AutoScalingComponent,
    aws_ecs_service_scaling::ECSServiceScalingComponent,
    aws_lambda_function::LambdaFunctionScalingComponent,
    azure_functions_app::AzureFunctionsAppScalingComponent,
    azure_vmss_autoscaling::VMSSAutoScalingComponent, gcp_mig_autoscaling::MIGAutoScalingComponent,
    google_cloud_functions_instance::CloudFunctionsInstanceScalingComponent,
    google_cloud_run_service::CloudRunServiceScalingComponent,
    k8s_deployment::K8sDeploymentScalingComponent,
};
use anyhow::Result;

// ScalingComponent can be used in multiple threads. So it needs to be Send + Sync.
#[async_trait]
pub trait ScalingComponent: Send + Sync {
    async fn apply(&self, params: HashMap<String, serde_json::Value>) -> Result<()>;
    fn get_scaling_component_kind(&self) -> &str;
    fn get_id(&self) -> &str;
}

//
// ScalingComponentManager
//
pub type SharedScalingComponentManager = Arc<RwLock<ScalingComponentManager>>;

#[derive(Default)]
pub struct ScalingComponentManager {
    scaling_components: HashMap<String, Box<dyn ScalingComponent>>,
}

impl ScalingComponentManager {
    pub fn new() -> Self {
        ScalingComponentManager {
            scaling_components: HashMap::new(),
        }
    }
    pub fn new_shared() -> SharedScalingComponentManager {
        Arc::new(RwLock::new(ScalingComponentManager::new()))
    }

    // Factory method to create a scaling component.
    fn create_scaling_component(
        &self,
        definition: &ScalingComponentDefinition,
    ) -> Result<Box<dyn ScalingComponent>> {
        // Get a value of metric and clone it.
        let cloned_defintion = definition.clone();
        match cloned_defintion.component_kind.as_str() {
            EC2AutoScalingComponent::SCALING_KIND => {
                Ok(Box::new(EC2AutoScalingComponent::new(cloned_defintion)))
            }
            ECSServiceScalingComponent::SCALING_KIND => {
                Ok(Box::new(ECSServiceScalingComponent::new(cloned_defintion)))
            }
            K8sDeploymentScalingComponent::SCALING_KIND => Ok(Box::new(
                K8sDeploymentScalingComponent::new(cloned_defintion),
            )),
            LambdaFunctionScalingComponent::SCALING_KIND => Ok(Box::new(
                LambdaFunctionScalingComponent::new(cloned_defintion),
            )),
            DynamoDbTableScalingComponent::SCALING_KIND => Ok(Box::new(
                DynamoDbTableScalingComponent::new(cloned_defintion),
            )),
            MIGAutoScalingComponent::SCALING_KIND => {
                Ok(Box::new(MIGAutoScalingComponent::new(cloned_defintion)))
            }
            VMSSAutoScalingComponent::SCALING_KIND => {
                Ok(Box::new(VMSSAutoScalingComponent::new(cloned_defintion)))
            }
            CloudFunctionsInstanceScalingComponent::SCALING_KIND => Ok(Box::new(
                CloudFunctionsInstanceScalingComponent::new(cloned_defintion),
            )),
            EMREC2AutoScalingComponent::SCALING_KIND => {
                Ok(Box::new(EMREC2AutoScalingComponent::new(cloned_defintion)))
            }
            CloudRunServiceScalingComponent::SCALING_KIND => Ok(Box::new(
                CloudRunServiceScalingComponent::new(cloned_defintion),
            )),
            AzureFunctionsAppScalingComponent::SCALING_KIND => Ok(Box::new(
                AzureFunctionsAppScalingComponent::new(cloned_defintion),
            )),
            _ => Err(anyhow::anyhow!("Unknown trigger kind")),
        }
    }

    pub fn add_definition(
        &mut self,
        scaling_component_definition: ScalingComponentDefinition,
    ) -> Result<()> {
        let scaling_component = self.create_scaling_component(&scaling_component_definition)?;
        self.add_scaling_component(scaling_component);
        Ok(())
    }

    pub fn add_definitions(
        &mut self,
        scaling_component_definitions: Vec<ScalingComponentDefinition>,
    ) -> Result<()> {
        for scaling_component_definition in scaling_component_definitions {
            self.add_definition(scaling_component_definition)?;
        }
        Ok(())
    }

    pub fn add_scaling_component(&mut self, scaling_component: Box<dyn ScalingComponent>) {
        self.scaling_components
            .insert(scaling_component.get_id().to_string(), scaling_component);
    }

    pub fn get_scaling_components(&self) -> &HashMap<String, Box<dyn ScalingComponent>> {
        &self.scaling_components
    }

    pub fn remove_all(&mut self) {
        self.scaling_components.clear();
    }

    pub fn get_scaling_component(&self, id: &str) -> Option<&Box<dyn ScalingComponent>> {
        self.scaling_components.get(id)
    }

    pub async fn apply_to(
        &self,
        id: &str,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        match self.scaling_components.get(id) {
            Some(scaling_component) => scaling_component.apply(params).await,
            None => Err(anyhow::anyhow!("Unknown scaling component kind")),
        }
    }
}

pub fn target_value_expression_regex_filter(
    target_value_expression: &str,
    target_value_key_array: Vec<String>,
) -> Vec<String> {
    let pattern = r"\$(".to_string() + &target_value_key_array.join("|") + r")";
    let re = regex::Regex::new(&pattern).unwrap();
    let mut result_vec = vec![];
    for mat in re.find_iter(target_value_expression) {
        let match_value = mat.as_str().to_string();
        if !result_vec.contains(&match_value) {
            result_vec.push(mat.as_str().to_string());
        }
    }
    result_vec
}

pub async fn get_target_value_result(
    context: rquickjs::AsyncContext,
    target_value_expression: &str,
    target_value_map: HashMap<String, i64>,
) -> Result<i64, anyhow::Error> {
    rquickjs::async_with!(context => |ctx| {
        target_value_map.iter().for_each(|(target_value_key, target_value_value)| {
            let _ = ctx.globals().set(
                target_value_key, target_value_value
            );
        })
    })
    .await;

    let target_value_result = rquickjs::async_with!(context => |ctx| {
        let Result::Ok(result) = ctx.eval::<i64, _>(target_value_expression) else {
            return Err(anyhow::anyhow!("Invalid target value"));
        };
        Ok(result)
    })
    .await;

    target_value_result
}

#[cfg(test)]
mod test {
    use super::*;
    use strum::IntoEnumIterator;
    use strum_macros::EnumIter;

    #[derive(Debug, EnumIter)]
    enum TestComponentTargetValue {
        Test1,
        Test2,
    }
    impl std::fmt::Display for TestComponentTargetValue {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                TestComponentTargetValue::Test1 => write!(f, "test1"),
                TestComponentTargetValue::Test2 => write!(f, "test2"),
            }
        }
    }

    #[test]
    fn test_target_value_expression_regex_filter() {
        let target_value_expression = "$test1 + 2 + $test2";
        let target_value_key_array = TestComponentTargetValue::iter()
            .map(|value| value.to_string())
            .collect::<Vec<String>>();
        assert_eq!(
            target_value_expression_regex_filter(
                target_value_expression,
                target_value_key_array.clone()
            ),
            vec!["$test1", "$test2"]
        );

        let target_value_expression2 = "1";
        assert!(target_value_expression_regex_filter(
            target_value_expression2,
            target_value_key_array
        )
        .is_empty());
    }

    #[tokio::test]
    async fn test_evaluation_target_value() {
        let Result::Ok(runtime) = rquickjs::AsyncRuntime::new() else {
            return;
        };
        let Result::Ok(context) = rquickjs::AsyncContext::full(&runtime).await else {
            return;
        };
        let target_value_key_array = TestComponentTargetValue::iter()
            .map(|value| value.to_string())
            .collect::<Vec<String>>();

        let target_value_expression = "($test1 * 2) + $test2";
        let mut target_value_map = HashMap::new();
        for target_value_key in
            target_value_expression_regex_filter(target_value_expression, target_value_key_array)
        {
            if target_value_key.eq(&format!("${}", TestComponentTargetValue::Test1)) {
                target_value_map.insert(target_value_key, 1);
            } else if target_value_key.eq(&format!("${}", TestComponentTargetValue::Test2)) {
                target_value_map.insert(target_value_key, 2);
            }
        }
        assert_eq!(
            get_target_value_result(
                context.clone(),
                target_value_expression,
                target_value_map.clone()
            )
            .await
            .unwrap(),
            4
        );

        let target_value_expression2 = "2 * 4";
        assert_eq!(
            get_target_value_result(
                context.clone(),
                target_value_expression2,
                target_value_map.clone()
            )
            .await
            .unwrap(),
            8
        );

        let target_value_expression3 = "4 * 4";
        assert_eq!(
            get_target_value_result(context.clone(), target_value_expression3, HashMap::new())
                .await
                .unwrap(),
            16
        );
    }
}
