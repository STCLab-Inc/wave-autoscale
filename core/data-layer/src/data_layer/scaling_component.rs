use super::DataLayer;
use crate::{
    types::object_kind::ObjectKind,
    values_map::{apply_values_map, get_values_map},
    ScalingComponentDefinition,
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

impl DataLayer {
    // Sync scaling components with the yaml - scaling components are all deleted and then added
    pub async fn sync_scaling_component_yaml(&self, yaml: &str) -> Result<()> {
        self.sync_scaling_component_yaml_for_unmatched_ids(yaml, true)
            .await
    }

    // Sync scaling components with the yaml - compare DB and yaml id, match id is ignore (no add)
    pub async fn sync_scaling_component_yaml_for_unmatched_ids(
        &self,
        yaml: &str,
        reset: bool,
    ) -> Result<()> {
        let deserializer = serde_yaml::Deserializer::from_str(yaml);
        let mut scaling_component_definitions: Vec<(ScalingComponentDefinition, String)> =
            Vec::new();

        let mut db_scaling_component_ids: HashMap<String, bool> = HashMap::new();
        if !reset {
            // search DB ScalingComponent Definitions.
            let db_all_scaling_components = self.get_all_scaling_components().await?;
            db_all_scaling_components
                .iter()
                .for_each(|scaling_component| {
                    db_scaling_component_ids.insert(scaling_component.id.clone(), true);
                });
        }

        for document in deserializer {
            // Get the yaml from the document
            let value = serde_yaml::Value::deserialize(document)?;
            let kind = value.get("kind").and_then(serde_yaml::Value::as_str);
            if kind.is_none() || kind.unwrap() != ObjectKind::ScalingComponent.to_string() {
                continue;
            }
            let parsed = serde_yaml::from_value::<ScalingComponentDefinition>(value.clone())?;
            let document_yaml = serde_yaml::to_string(&value)?;
            // match id is ignore (no add)
            if !reset && db_scaling_component_ids.contains_key(parsed.id.as_str()) {
                continue;
            }
            scaling_component_definitions.push((parsed, document_yaml));
        }

        if reset {
            // Remove all scaling components
            self.delete_all_scaling_components().await?;
        }

        // Add scaling components
        self.add_scaling_components(scaling_component_definitions)
            .await
    }

    // Add multiple scaling components to the database
    pub async fn add_scaling_components(
        &self,
        scaling_components: Vec<(ScalingComponentDefinition, String)>,
    ) -> Result<()> {
        // Define a pool variable that is a trait to pass to the execute function
        for (scaling_component, yaml) in scaling_components {
            let metadata_string = serde_json::to_string(&scaling_component.metadata).unwrap();
            let query_string =
                "INSERT INTO scaling_component (db_id, id, component_kind, metadata, enabled, yaml, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT (id) DO UPDATE SET (metadata, enabled, yaml, updated_at) = ($9,$10,$11,$12)";
            let id = Uuid::new_v4().to_string();
            let updated_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            let result = sqlx::query(query_string)
                // Values for insert
                .bind(id)
                .bind(scaling_component.id)
                .bind(scaling_component.component_kind)
                .bind(metadata_string.clone())
                .bind(scaling_component.enabled)
                .bind(yaml.clone())
                .bind(updated_at.clone())
                .bind(updated_at.clone())
                // Values for update
                .bind(metadata_string.clone())
                .bind(scaling_component.enabled)
                .bind(yaml)
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

        let variable_mapper_data = get_values_map();

        for row in result {
            let mut metadata = HashMap::new();
            if let Ok(metadata_str) = row.try_get::<&str, _>("metadata") {
                let metadata_json =
                    apply_values_map(metadata_str.to_string(), &variable_mapper_data);
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

    // Get all scaling component yamls from the database
    pub async fn get_scaling_component_yamls(&self) -> Result<Vec<String>> {
        let mut scaling_component_yamls: Vec<String> = Vec::new();
        let query_string = "SELECT yaml FROM scaling_component";
        let result = sqlx::query(query_string).fetch_all(&self.pool).await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();
        for row in result {
            scaling_component_yamls.push(row.try_get("yaml")?);
        }
        Ok(scaling_component_yamls)
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

    // Delete all scaling components from the database
    pub async fn delete_all_scaling_components(&self) -> Result<()> {
        let query_string = "DELETE FROM scaling_component";
        let result = sqlx::query(query_string).execute(&self.pool).await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        Ok(())
    }

    // // Get a scaling component from the database
    // pub async fn get_scaling_component_by_id(
    //     &self,
    //     db_id: String,
    // ) -> Result<ScalingComponentDefinition> {
    //     let query_string =
    //         "SELECT db_id, id, component_kind, metadata, enabled FROM scaling_component WHERE db_id=$1";
    //     let result = sqlx::query(query_string)
    //         .bind(db_id)
    //         .fetch_one(&self.pool)
    //         .await;
    //     if result.is_err() {
    //         return Err(anyhow!(result.err().unwrap().to_string()));
    //     }
    //     let result = result.unwrap();
    //     let scaling_component = ScalingComponentDefinition {
    //         kind: ObjectKind::ScalingComponent,
    //         db_id: result.get("db_id"),
    //         id: result.get("id"),
    //         component_kind: result.get("component_kind"),
    //         metadata: serde_json::from_str(result.get("metadata")).unwrap(),
    //         enabled: result.get("enabled"),
    //     };
    //     Ok(scaling_component)
    // }

    // // Delete a scaling component
    // pub async fn delete_scaling_component(&self, db_id: String) -> Result<AnyQueryResult> {
    //     let query_string = "DELETE FROM scaling_component WHERE db_id=$1";
    //     let result = sqlx::query(query_string)
    //         .bind(db_id)
    //         .execute(&self.pool)
    //         .await;
    //     if result.is_err() {
    //         return Err(anyhow!(result.err().unwrap().to_string()));
    //     }
    //     let result = result.unwrap();
    //     if result.rows_affected() == 0 {
    //         return Err(anyhow!("No rows affected"));
    //     }
    //     Ok(result)
    // }
    // // Update a scaling component in the database
    // pub async fn update_scaling_component(
    //     &self,
    //     scaling_component: ScalingComponentDefinition,
    // ) -> Result<AnyQueryResult> {
    //     let metadata_string = serde_json::to_string(&scaling_component.metadata).unwrap();
    //     let query_string =
    //         "UPDATE scaling_component SET id=$1, component_kind=$2, metadata=$3, enabled=$4, updated_at=$5 WHERE db_id=$6";
    //     let updated_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    //     let result = sqlx::query(query_string)
    //         // SET
    //         .bind(scaling_component.id)
    //         .bind(scaling_component.component_kind)
    //         .bind(metadata_string)
    //         .bind(scaling_component.enabled)
    //         .bind(updated_at)
    //         // WHERE
    //         .bind(scaling_component.db_id)
    //         .execute(&self.pool)
    //         .await;
    //     if result.is_err() {
    //         return Err(anyhow!(result.err().unwrap().to_string()));
    //     }
    //     let result = result.unwrap();
    //     if result.rows_affected() == 0 {
    //         return Err(anyhow!("No rows affected"));
    //     }
    //     Ok(result)
    // }
}

#[cfg(test)]
mod tests {
    use super::DataLayer;
    use crate::data_layer::tests::{get_data_layer_with_postgres, get_data_layer_with_sqlite};
    use tracing_test::traced_test;

    #[tokio::test]
    #[traced_test]
    async fn test_sync_scaling_component_yaml() {
        let data_layer = get_data_layer_with_sqlite().await;
        test_sync_scaling_component_yaml_with_data_layer(data_layer).await;

        let data_layer = get_data_layer_with_postgres().await;
        test_sync_scaling_component_yaml_with_data_layer(data_layer).await;
    }
    async fn test_sync_scaling_component_yaml_with_data_layer(data_layer: DataLayer) {
        //
        // 1. Initial sync
        //
        let yaml = r#"
kind: ScalingComponent
id: test_scaling_component_1
component_kind: test_component_kind_1
metadata:
    test_metadata_key_1: test_metadata_value_1
    test_metadata_key_2: test_metadata_value_2
enabled: true
        "#;
        let result = data_layer.sync_scaling_component_yaml(yaml).await;
        assert!(result.is_ok());

        // Validate the scaling component
        let scaling_components = data_layer.get_all_scaling_components().await.unwrap();
        assert_eq!(scaling_components.len(), 1);
        assert_eq!(scaling_components[0].id, "test_scaling_component_1");
        assert_eq!(
            scaling_components[0].component_kind,
            "test_component_kind_1"
        );
        assert_eq!(
            scaling_components[0]
                .metadata
                .get("test_metadata_key_1")
                .unwrap(),
            "test_metadata_value_1"
        );

        // JSON
        let scaling_components_json = data_layer.get_all_scaling_components_json().await.unwrap();
        assert_eq!(scaling_components_json.len(), 1);
        assert_eq!(scaling_components_json[0]["id"], "test_scaling_component_1");
        assert_eq!(
            scaling_components_json[0]["component_kind"],
            "test_component_kind_1"
        );
        assert_eq!(
            scaling_components_json[0]["metadata"]["test_metadata_key_1"],
            "test_metadata_value_1"
        );

        // YAML
        let scaling_component_yamls = data_layer.get_scaling_component_yamls().await.unwrap();
        assert_eq!(scaling_component_yamls.len(), 1);
        let scaling_component_yaml_1 = scaling_component_yamls[0].clone();
        let scaling_component_yaml_1: serde_yaml::Value =
            serde_yaml::from_str(scaling_component_yaml_1.as_str()).unwrap();
        assert_eq!(scaling_component_yaml_1["id"], "test_scaling_component_1");
        assert_eq!(
            scaling_component_yaml_1["component_kind"],
            "test_component_kind_1"
        );
        assert_eq!(
            scaling_component_yaml_1["metadata"]["test_metadata_key_1"],
            "test_metadata_value_1"
        );

        //
        // 2. Second sync
        //
        let yaml = r#"
kind: ScalingComponent
id: test_scaling_component_2
component_kind: test_component_kind_2
metadata:
    test_metadata_key_3: test_metadata_value_3
    test_metadata_key_4: test_metadata_value_4
enabled: true
---
kind: ScalingComponent
id: test_scaling_component_3
component_kind: test_component_kind_3
metadata:
    test_metadata_key_5: test_metadata_value_5
    test_metadata_key_6: test_metadata_value_6
enabled: true
        "#;
        let result = data_layer.sync_scaling_component_yaml(yaml).await;
        if result.is_err() {
            println!("ERROR: {:?}", result.err().unwrap());
        }

        // Validate the scaling components
        let scaling_components = data_layer.get_all_scaling_components().await.unwrap();
        assert_eq!(scaling_components.len(), 2);
        assert_eq!(scaling_components[0].id, "test_scaling_component_2");
        assert_eq!(
            scaling_components[0].component_kind,
            "test_component_kind_2"
        );
        assert_eq!(
            scaling_components[0]
                .metadata
                .get("test_metadata_key_3")
                .unwrap(),
            "test_metadata_value_3"
        );
        assert_eq!(scaling_components[1].id, "test_scaling_component_3");
        assert_eq!(
            scaling_components[1].component_kind,
            "test_component_kind_3"
        );
        assert_eq!(
            scaling_components[1]
                .metadata
                .get("test_metadata_key_5")
                .unwrap(),
            "test_metadata_value_5"
        );

        // JSON
        let scaling_components_json = data_layer.get_all_scaling_components_json().await.unwrap();
        assert_eq!(scaling_components_json.len(), 2);
        assert_eq!(scaling_components_json[0]["id"], "test_scaling_component_2");
        assert_eq!(
            scaling_components_json[0]["component_kind"],
            "test_component_kind_2"
        );
        assert_eq!(
            scaling_components_json[0]["metadata"]["test_metadata_key_3"],
            "test_metadata_value_3"
        );
        assert_eq!(scaling_components_json[1]["id"], "test_scaling_component_3");
        assert_eq!(
            scaling_components_json[1]["component_kind"],
            "test_component_kind_3"
        );
        assert_eq!(
            scaling_components_json[1]["metadata"]["test_metadata_key_5"],
            "test_metadata_value_5"
        );

        // YAML
        let scaling_component_yamls = data_layer.get_scaling_component_yamls().await.unwrap();
        assert_eq!(scaling_component_yamls.len(), 2);
        let scaling_component_yaml_2 = scaling_component_yamls[0].clone();
        let scaling_component_yaml_2: serde_yaml::Value =
            serde_yaml::from_str(scaling_component_yaml_2.as_str()).unwrap();
        assert_eq!(scaling_component_yaml_2["id"], "test_scaling_component_2");
        assert_eq!(
            scaling_component_yaml_2["component_kind"],
            "test_component_kind_2"
        );
        assert_eq!(
            scaling_component_yaml_2["metadata"]["test_metadata_key_3"],
            "test_metadata_value_3"
        );
        let scaling_component_yaml_3 = scaling_component_yamls[1].clone();
        let scaling_component_yaml_3: serde_yaml::Value =
            serde_yaml::from_str(scaling_component_yaml_3.as_str()).unwrap();
        assert_eq!(scaling_component_yaml_3["id"], "test_scaling_component_3");
        assert_eq!(
            scaling_component_yaml_3["component_kind"],
            "test_component_kind_3"
        );
        assert_eq!(
            scaling_component_yaml_3["metadata"]["test_metadata_key_5"],
            "test_metadata_value_5"
        );
    }
}
