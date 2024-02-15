use super::DataLayer;
use crate::{
    types::object_kind::ObjectKind,
    values_map::{apply_values_map, get_values_map},
    MetricDefinition,
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

impl DataLayer {
    pub async fn sync_metric_yaml(&self, yaml: &str) -> Result<()> {
        let deserializer = serde_yaml::Deserializer::from_str(yaml);
        let mut metric_definitions: Vec<(MetricDefinition, String)> = Vec::new();

        for document in deserializer {
            // Get the yaml from the document
            let value = serde_yaml::Value::deserialize(document)?;
            let kind = value.get("kind").and_then(serde_yaml::Value::as_str);
            if kind.is_none() || kind.unwrap() != ObjectKind::Metric.to_string() {
                continue;
            }
            let parsed = serde_yaml::from_value::<MetricDefinition>(value.clone())?;
            let document_yaml = serde_yaml::to_string(&value)?;
            metric_definitions.push((parsed, document_yaml));
        }

        // Remove metrics
        self.delete_all_metrics().await?;

        // Add metrics
        self.add_metrics(metric_definitions).await
    }
    // Add multiple metrics to the database
    pub async fn add_metrics(&self, metrics: Vec<(MetricDefinition, String)>) -> Result<()> {
        // Define a pool variable that is a trait to pass to the execute function
        for (metric, yaml) in metrics {
            let metadata_string = serde_json::to_string(&metric.metadata).unwrap();
            let query_string =
                "INSERT INTO metric (db_id, id, collector, metadata, enabled, yaml, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT (id) DO UPDATE SET (collector, metadata, enabled, yaml, updated_at) = ($9,$10,$11,$12,$13)";
            let db_id = Uuid::new_v4().to_string();
            let updated_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            let result = sqlx::query(query_string)
                // Values for insert
                .bind(db_id)
                .bind(metric.id.to_lowercase())
                .bind(metric.collector.to_lowercase())
                .bind(metadata_string.clone())
                .bind(metric.enabled)
                .bind(yaml.clone())
                .bind(updated_at.clone())
                .bind(updated_at.clone())
                // Values for update
                .bind(metric.collector.to_lowercase())
                .bind(metadata_string.clone())
                .bind(metric.enabled)
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
    // Get all metrics from the database
    pub async fn get_all_metrics(&self) -> Result<Vec<MetricDefinition>> {
        let mut metrics: Vec<MetricDefinition> = Vec::new();
        let query_string = "SELECT db_id, id, collector, metadata, enabled FROM metric";
        let result = sqlx::query(query_string).fetch_all(&self.pool).await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();

        let variable_mapper_data = get_values_map();

        for row in result {
            let metadata = match row.try_get::<Option<&str>, _>("metadata") {
                Ok(Some(metadata_str)) => {
                    apply_values_map(metadata_str.to_string(), &variable_mapper_data)
                        .map_err(|e| anyhow!("Error in execute_variable_mapper: {}", e))?
                }
                Ok(None) => serde_json::Value::Null.to_string(),
                Err(e) => return Err(anyhow!("Error getting metadata: {}", e)),
            };

            metrics.push(MetricDefinition {
                kind: ObjectKind::Metric,
                db_id: row.try_get("db_id")?,
                id: row.try_get("id")?,
                collector: row.try_get("collector")?,
                metadata: serde_json::from_str(metadata.as_str()).unwrap(),
                enabled: row.try_get("enabled")?,
            });
        }
        Ok(metrics)
    }

    // Get all metrics that are enabled
    pub async fn get_enabled_metrics(&self) -> Result<Vec<MetricDefinition>> {
        let metrics = self.get_all_metrics().await?;
        let metrics = metrics
            .into_iter()
            .filter(|metric| metric.enabled)
            .collect::<Vec<MetricDefinition>>();
        Ok(metrics)
    }

    // Get all metric yamls from the database
    pub async fn get_metric_yamls(&self) -> Result<Vec<String>> {
        let query_string = "SELECT yaml FROM metric";
        let result = sqlx::query(query_string).fetch_all(&self.pool).await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();
        let mut yamls: Vec<String> = Vec::new();
        for row in result {
            yamls.push(row.try_get("yaml")?);
        }
        Ok(yamls)
    }

    // Get all metrics json from the database
    pub async fn get_all_metrics_json(&self) -> Result<Vec<serde_json::Value>> {
        let mut metrics: Vec<serde_json::Value> = Vec::new();
        let query_string =
            "SELECT db_id, id, collector, metadata, enabled, created_at, updated_at FROM metric";
        let result = sqlx::query(query_string).fetch_all(&self.pool).await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();
        for row in result {
            let metric = json!({
                "kind": ObjectKind::Metric,
                "db_id": row.try_get::<String, _>("db_id")?,
                "id": row.try_get::<String, _>("id")?,
                "collector": row.try_get::<String, _>("collector")?,
                "metadata": serde_json::from_str::<serde_json::Value>(row.try_get::<String, _>("metadata")?.as_str())?,
                "enabled": row.try_get::<bool, _>("enabled")?,
                "created_at": row.try_get::<Option<String>, _>("created_at")?,
                "updated_at": row.try_get::<Option<String>, _>("updated_at")?,
            });
            metrics.push(metric);
        }
        Ok(metrics)
    }

    // Delete all metrics from the database
    pub async fn delete_all_metrics(&self) -> Result<()> {
        let query_string = "DELETE FROM metric";
        let result = sqlx::query(query_string).execute(&self.pool).await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        Ok(())
    }

    // // Get a metric from the database
    // pub async fn get_metric_by_id(&self, db_id: String) -> Result<Option<MetricDefinition>> {
    //     let query_string =
    //         "SELECT db_id, id, collector, metadata, enabled FROM metric WHERE db_id=$1";
    //     let result = sqlx::query(query_string)
    //         .bind(db_id)
    //         // Do not use fetch_one because it expects exact one result. If not, it will return an error
    //         .fetch_all(&self.pool)
    //         .await;
    //     if result.is_err() {
    //         return Err(anyhow!(result.err().unwrap().to_string()));
    //     }
    //     let result = result.unwrap();
    //     if result.is_empty() {
    //         return Ok(None);
    //     }
    //     let Some(row) = result.get(0) else {
    //         return Ok(None);
    //     };
    //     let mut metadata = HashMap::new();
    //     if let Ok(metadata_str) = row.try_get::<&str, _>("metadata") {
    //         let metadata_json = serde_json::from_str(metadata_str);
    //         if metadata_json.is_ok() {
    //             metadata = metadata_json.unwrap();
    //         }
    //     }
    //     let metric = MetricDefinition {
    //         kind: ObjectKind::Metric,
    //         db_id: row.try_get("db_id")?,
    //         id: row.try_get("id")?,
    //         collector: row.try_get("collector")?,
    //         metadata,
    //         enabled: row.try_get("enabled")?,
    //     };
    //     Ok(Some(metric))
    // }
    // // Delete a metric
    // pub async fn delete_metric(&self, db_id: String) -> Result<AnyQueryResult> {
    //     let query_string = "DELETE FROM metric WHERE db_id=$1";
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
    // // Update a metric in the database
    // pub async fn update_metric(&self, metric: MetricDefinition) -> Result<AnyQueryResult> {
    //     let metadata_string = serde_json::to_string(&metric.metadata).unwrap();
    //     let query_string =
    //         "UPDATE metric SET id=$1, collector=$2, metadata=$3, updated_at=$4, enabled=$5 WHERE db_id=$6";
    //     let updated_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    //     let result = sqlx::query(query_string)
    //         // SET
    //         .bind(metric.id)
    //         .bind(metric.collector)
    //         .bind(metadata_string)
    //         .bind(updated_at)
    //         .bind(metric.enabled)
    //         // WHERE
    //         .bind(metric.db_id)
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
    async fn test_sync_metric_yaml() {
        let data_layer = get_data_layer_with_sqlite().await;
        test_sync_metric_yaml_with_data_layer(data_layer).await;

        let data_layer = get_data_layer_with_postgres().await;
        test_sync_metric_yaml_with_data_layer(data_layer).await;
    }
    async fn test_sync_metric_yaml_with_data_layer(data_layer: DataLayer) {
        //
        // 1. Initial sync
        //
        let metric_definition_yaml = r#"
kind: Metric
id: metric_test_id
collector: vector
metadata:
  user_key: user_value
enabled: true"#;

        // sync metrics
        let sync_metrics_result = data_layer.sync_metric_yaml(metric_definition_yaml).await;
        assert!(sync_metrics_result.is_ok());

        // Validate metrics
        let metrics = data_layer.get_all_metrics().await;
        assert!(metrics.is_ok());
        let metrics = metrics.unwrap();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics.get(0).unwrap().id, "metric_test_id");

        // JSON
        let metrics_json = data_layer.get_all_metrics_json().await;
        metrics_json.unwrap().iter().for_each(|metric| {
            if metric.get("id").unwrap() == "metric_test_id" {
                assert_eq!(metric.get("enabled").unwrap(), &true);
            }
        });

        // YAML
        let metrics_yaml = data_layer.get_metric_yamls().await;
        assert!(metrics_yaml.is_ok());
        let metrics_yaml = metrics_yaml.unwrap();
        assert_eq!(metrics_yaml.len(), 1);
        let metrics_yaml_1 = metrics_yaml.get(0).unwrap();
        let metrics_yaml_1: serde_yaml::Value = serde_yaml::from_str(metrics_yaml_1).unwrap();
        assert_eq!(metrics_yaml_1["id"], "metric_test_id");
        assert_eq!(metrics_yaml_1["enabled"], true);

        //
        // 2. Second sync
        //
        let metric_definition_yaml_1 = r#"
kind: Metric
id: metric_test_id
collector: vector
metadata:
    user_key: user_value
enabled: false"#;

        let metric_definition_yaml_2 = r#"
kind: Metric
id: metric_test_id_2
collector: vector
metadata:
    user_key: user_value
enabled: true"#;

        let metric_definition_yaml = format!(
            "{}\n---\n{}",
            metric_definition_yaml_1, metric_definition_yaml_2
        );
        let metric_definition_yaml = metric_definition_yaml.as_str();

        // sync metrics
        let sync_metrics_result = data_layer.sync_metric_yaml(metric_definition_yaml).await;
        assert!(sync_metrics_result.is_ok());

        // Validate metrics
        let metrics = data_layer.get_all_metrics().await;
        assert!(metrics.is_ok());
        let metrics = metrics.unwrap();
        assert_eq!(metrics.len(), 2);
        assert_eq!(metrics.get(0).unwrap().id, "metric_test_id");
        assert_eq!(metrics.get(1).unwrap().id, "metric_test_id_2");

        // JSON
        let metrics_json = data_layer.get_all_metrics_json().await;
        metrics_json.unwrap().iter().for_each(|metric| {
            if metric.get("id").unwrap() == "metric_test_id" {
                assert_eq!(metric.get("enabled").unwrap(), &false);
            }
            if metric.get("id").unwrap() == "metric_test_id_2" {
                assert_eq!(metric.get("enabled").unwrap(), &true);
            }
        });

        // YAML
        let metrics_yaml = data_layer.get_metric_yamls().await;
        assert!(metrics_yaml.is_ok());
        let metrics_yaml = metrics_yaml.unwrap();
        assert_eq!(metrics_yaml.len(), 2);
        let metrics_yaml_1 = metrics_yaml.get(0).unwrap();
        let metrics_yaml_1: serde_yaml::Value = serde_yaml::from_str(metrics_yaml_1).unwrap();
        assert_eq!(metrics_yaml_1["id"], "metric_test_id");
        assert_eq!(metrics_yaml_1["enabled"], false);
        let metrics_yaml_2 = metrics_yaml.get(1).unwrap();
        let metrics_yaml_2: serde_yaml::Value = serde_yaml::from_str(metrics_yaml_2).unwrap();
        assert_eq!(metrics_yaml_2["id"], "metric_test_id_2");
        assert_eq!(metrics_yaml_2["enabled"], true);
    }
}
