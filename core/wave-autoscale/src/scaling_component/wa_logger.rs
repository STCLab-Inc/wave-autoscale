use super::ScalingComponent;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use data_layer::ScalingComponentDefinition;
use serde_json::Value;
use std::collections::HashMap;
use tracing::info;

pub struct WALoggerComponent {
    definition: ScalingComponentDefinition,
}

impl WALoggerComponent {
    // Static variables
    pub const SCALING_KIND: &'static str = "wa-logger";

    // Functions
    pub fn new(definition: ScalingComponentDefinition) -> Self {
        WALoggerComponent { definition }
    }
}

#[async_trait]
impl ScalingComponent for WALoggerComponent {
    fn get_scaling_component_kind(&self) -> &str {
        &self.definition.component_kind
    }
    fn get_id(&self) -> &str {
        &self.definition.id
    }
    async fn apply(
        &self,
        params: HashMap<String, Value>,
        _context: rquickjs::AsyncContext,
    ) -> Result<HashMap<String, Value>> {
        info!("[wa-logger] params: {:?}", params);
        Ok(params)
    }
}

#[cfg(test)]
mod test {
    use super::WALoggerComponent;
    use crate::scaling_component::test::get_rquickjs_context;
    use crate::scaling_component::ScalingComponent;
    use data_layer::ScalingComponentDefinition;
    use std::collections::HashMap;

    // Purpose of the test is call apply function and fail test. just consists of test forms only.
    #[tokio::test]
    async fn test_apply() {
        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling_id"),
            component_kind: String::from("wa-logger"),
            metadata: HashMap::new(),
            ..Default::default()
        };

        let params = HashMap::from([
            (
                "key1".to_string(),
                serde_json::Value::String("value1".to_string()),
            ),
            (
                "key2".to_string(),
                serde_json::Value::Number(serde_json::Number::from(100)),
            ),
        ]);
        let scaling_component = WALoggerComponent::new(scaling_definition)
            .apply(params, get_rquickjs_context().await)
            .await;
        assert!(scaling_component.is_ok());
    }
}
