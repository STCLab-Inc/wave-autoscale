use super::DataLayer;
use crate::{
    types::object_kind::ObjectKind,
    variable_mapper::{execute_variable_mapper, get_variable_mapper},
    ScalingComponentDefinition,
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde_json::json;
use sqlx::{any::AnyQueryResult, Row};
use std::collections::HashMap;
use uuid::Uuid;

impl DataLayer {
    // Add multiple scaling components to the database
    pub async fn add_scaling_components(
        &self,
        scaling_components: Vec<ScalingComponentDefinition>,
    ) -> Result<()> {
        // Define a pool variable that is a trait to pass to the execute function
        for scaling_component in scaling_components {
            let metadata_string = serde_json::to_string(&scaling_component.metadata).unwrap();
            let query_string =
                "INSERT INTO scaling_component (db_id, id, component_kind, metadata, enabled, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT (id) DO UPDATE SET (metadata, enabled, updated_at) = ($8,$9,$10)";
            let id = Uuid::new_v4().to_string();
            let updated_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            let result = sqlx::query(query_string)
                // Values for insert
                .bind(id)
                .bind(scaling_component.id)
                .bind(scaling_component.component_kind)
                .bind(metadata_string.clone())
                .bind(scaling_component.enabled)
                .bind(updated_at.clone())
                .bind(updated_at.clone())
                // Values for update
                .bind(metadata_string.clone())
                .bind(scaling_component.enabled)
                .bind(updated_at.clone())
                // Run
                .execute(&self.pool)
                .await;
            if result.is_err() {
                return Err(anyhow!(result.err().unwrap().to_string()));
            }
        }
        Ok(())
    }
    // Get all scaling components from the database
    pub async fn get_all_scaling_components(&self) -> Result<Vec<ScalingComponentDefinition>> {
        let mut scaling_components: Vec<ScalingComponentDefinition> = Vec::new();
        let query_string =
            "SELECT db_id, id, component_kind, metadata, enabled FROM scaling_component";
        let result = sqlx::query(query_string).fetch_all(&self.pool).await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();

        let variable_mapper_data = get_variable_mapper();

        for row in result {
            let mut metadata = HashMap::new();
            if let Ok(metadata_str) = row.try_get::<&str, _>("metadata") {
                let metadata_json =
                    execute_variable_mapper(metadata_str.to_string(), &variable_mapper_data);
                if metadata_json.is_ok() {
                    let json = serde_json::from_str(metadata_json.unwrap().as_str());
                    if json.is_ok() {
                        metadata = json.unwrap();
                    }
                }
            }

            scaling_components.push(ScalingComponentDefinition {
                kind: ObjectKind::ScalingComponent,
                db_id: row.try_get("db_id")?,
                id: row.try_get("id")?,
                component_kind: row.try_get("component_kind")?,
                metadata,
                enabled: row.try_get("enabled")?,
            });
        }
        Ok(scaling_components)
    }
    // Get enabled scaling components
    pub async fn get_enabled_scaling_components(&self) -> Result<Vec<ScalingComponentDefinition>> {
        let scaling_components = self.get_all_scaling_components().await?;
        let scaling_components = scaling_components
            .into_iter()
            .filter(|scaling_component| scaling_component.enabled)
            .collect::<Vec<ScalingComponentDefinition>>();
        Ok(scaling_components)
    }
    // Get all scaling components json from the database
    pub async fn get_all_scaling_components_json(&self) -> Result<Vec<serde_json::Value>> {
        let mut scaling_components: Vec<serde_json::Value> = Vec::new();
        let query_string = "SELECT db_id, id, component_kind, metadata, enabled, created_at, updated_at FROM scaling_component";
        let result = sqlx::query(query_string).fetch_all(&self.pool).await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();
        for row in result {
            let scaling_component = json!({
                "kind": ObjectKind::ScalingComponent,
                "db_id": row.try_get::<String, _>("db_id")?,
                "id": row.try_get::<String, _>("id")?,
                "component_kind": row.try_get::<String, _>("component_kind")?,
                "metadata": serde_json::from_str::<serde_json::Value>(row.try_get::<String, _>("metadata")?.as_str())?,
                "enabled": row.try_get::<bool, _>("enabled")?,
                "created_at": row.try_get::<Option<String>, _>("created_at")?,
                "updated_at": row.try_get::<Option<String>, _>("updated_at")?,
            });
            scaling_components.push(scaling_component);
        }
        Ok(scaling_components)
    }
    // Get a scaling component from the database
    pub async fn get_scaling_component_by_id(
        &self,
        db_id: String,
    ) -> Result<ScalingComponentDefinition> {
        let query_string =
            "SELECT db_id, id, component_kind, metadata, enabled FROM scaling_component WHERE db_id=$1";
        let result = sqlx::query(query_string)
            .bind(db_id)
            .fetch_one(&self.pool)
            .await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();
        let scaling_component = ScalingComponentDefinition {
            kind: ObjectKind::ScalingComponent,
            db_id: result.get("db_id"),
            id: result.get("id"),
            component_kind: result.get("component_kind"),
            metadata: serde_json::from_str(result.get("metadata")).unwrap(),
            enabled: result.get("enabled"),
        };
        Ok(scaling_component)
    }
    // Delete all scaling components from the database
    pub async fn delete_all_scaling_components(&self) -> Result<()> {
        let query_string = "DELETE FROM scaling_component";
        let result = sqlx::query(query_string).execute(&self.pool).await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        Ok(())
    }
    // Delete a scaling component
    pub async fn delete_scaling_component(&self, db_id: String) -> Result<AnyQueryResult> {
        let query_string = "DELETE FROM scaling_component WHERE db_id=$1";
        let result = sqlx::query(query_string)
            .bind(db_id)
            .execute(&self.pool)
            .await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();
        if result.rows_affected() == 0 {
            return Err(anyhow!("No rows affected"));
        }
        Ok(result)
    }
    // Update a scaling component in the database
    pub async fn update_scaling_component(
        &self,
        scaling_component: ScalingComponentDefinition,
    ) -> Result<AnyQueryResult> {
        let metadata_string = serde_json::to_string(&scaling_component.metadata).unwrap();
        let query_string =
            "UPDATE scaling_component SET id=$1, component_kind=$2, metadata=$3, enabled=$4, updated_at=$5 WHERE db_id=$6";
        let updated_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let result = sqlx::query(query_string)
            // SET
            .bind(scaling_component.id)
            .bind(scaling_component.component_kind)
            .bind(metadata_string)
            .bind(scaling_component.enabled)
            .bind(updated_at)
            // WHERE
            .bind(scaling_component.db_id)
            .execute(&self.pool)
            .await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();
        if result.rows_affected() == 0 {
            return Err(anyhow!("No rows affected"));
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::DataLayer;
    use super::*;
    use crate::data_layer::tests::{get_data_layer_with_postgres, get_data_layer_with_sqlite};
    use tracing_test::traced_test;
    use ulid::Ulid;

    #[tokio::test]
    #[traced_test]
    async fn test_get_all_scaling_components_json() {
        let data_layer = get_data_layer_with_sqlite().await;
        test_get_all_scaling_components_json_with_data_layer(data_layer).await;

        let data_layer = get_data_layer_with_postgres().await;
        test_get_all_scaling_components_json_with_data_layer(data_layer).await;
    }
    async fn test_get_all_scaling_components_json_with_data_layer(data_layer: DataLayer) {
        let scaling_component_definition = ScalingComponentDefinition {
            kind: ObjectKind::ScalingComponent,
            db_id: Ulid::new().to_string(),
            id: "scaling_component_test_id".to_string(),
            component_kind: "test_component_kind".to_string(),
            metadata: HashMap::new(),
            enabled: true,
        };
        // add scaling_components
        let add_scaling_components_result = data_layer
            .add_scaling_components(vec![scaling_component_definition.clone()])
            .await;
        assert!(add_scaling_components_result.is_ok());

        // read scaling_components
        let scaling_components = data_layer.get_all_scaling_components_json().await;
        scaling_components
            .unwrap()
            .iter()
            .for_each(|scaling_component| {
                if scaling_component.get("id").unwrap() == "scaling_component_test_id" {
                    assert_eq!(scaling_component.get("enabled").unwrap(), &true);
                }
            });
    }
}
