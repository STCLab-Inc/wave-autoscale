use super::DataLayer;
use crate::types::autoscaling_history_definition::AutoscalingHistoryDefinition;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::Row;
use ulid::Ulid;

impl DataLayer {
    // Add AutoscalingHistory to the database
    pub async fn add_autoscaling_history(
        &self,
        autoscaling_history: AutoscalingHistoryDefinition,
    ) -> Result<()> {
        let query_string = "INSERT INTO autoscaling_history (id, plan_db_id, plan_id, plan_item_json, metric_values_json, metadata_values_json, fail_message) VALUES ($1,$2,$3,$4,$5,$6,$7)";
        let id = Ulid::new().to_string();
        let result = sqlx::query(query_string)
            // INTO
            .bind(id)
            .bind(autoscaling_history.plan_db_id)
            .bind(autoscaling_history.plan_id)
            .bind(autoscaling_history.plan_item_json)
            .bind(autoscaling_history.metric_values_json)
            .bind(autoscaling_history.metadata_values_json)
            .bind(autoscaling_history.fail_message)
            .execute(&self.pool)
            .await;

        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        Ok(())
    }
    // Get an AutoscalingHistory by plan_id from the database
    pub async fn get_autoscaling_history_by_plan_id(
        &self,
        plan_id: String,
    ) -> Result<Vec<AutoscalingHistoryDefinition>> {
        let mut autoscaling_history: Vec<AutoscalingHistoryDefinition> = Vec::new();
        let query_string = "SELECT id, plan_db_id, plan_id, plan_item_json, metric_values_json, metadata_values_json, fail_message FROM autoscaling_history WHERE plan_id=$1";
        let result = sqlx::query(query_string)
            .bind(plan_id)
            .fetch_all(&self.pool)
            .await;

        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();

        for row in result {
            autoscaling_history.push(AutoscalingHistoryDefinition {
                id: row.try_get("id")?,
                plan_db_id: row.try_get("plan_db_id")?,
                plan_id: row.try_get("plan_id")?,
                plan_item_json: row.try_get("plan_item_json")?,
                metric_values_json: row.try_get("metric_values_json")?,
                metadata_values_json: row.try_get("metadata_values_json")?,
                fail_message: row.try_get("fail_message")?,
            });
        }
        Ok(autoscaling_history)
    }
    // Get AutoscalingHistory by from and to date from the database
    pub async fn get_autoscaling_history_by_date(
        &self,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
    ) -> Result<Vec<AutoscalingHistoryDefinition>> {
        let mut autoscaling_history: Vec<AutoscalingHistoryDefinition> = Vec::new();
        // Convert from and to date to Ulid.
        // e.g. 2021-01-01 00:00:00.000 -> 01F8ZQZ1Z0Z000000000000000
        let from = Ulid::from_parts(from_date.timestamp_millis() as u64, 0).to_string();
        // e.g. 2021-01-01 00:00:00.000 + 0.001 -> 01F8ZQZ1Z0Z000000000000000
        let to_date = to_date + chrono::Duration::milliseconds(1);
        let to = Ulid::from_parts(to_date.timestamp_millis() as u64, 0).to_string();

        // Query
        let query_string = "SELECT id, plan_db_id, plan_id, plan_item_json, metric_values_json, metadata_values_json, fail_message FROM autoscaling_history WHERE id BETWEEN $1 AND $2";
        let result = sqlx::query(query_string)
            .bind(from)
            .bind(to)
            .fetch_all(&self.pool)
            .await;

        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();

        for row in result {
            autoscaling_history.push(AutoscalingHistoryDefinition {
                id: row.try_get("id")?,
                plan_db_id: row.try_get("plan_db_id")?,
                plan_id: row.try_get("plan_id")?,
                plan_item_json: row.try_get("plan_item_json")?,
                metric_values_json: row.try_get("metric_values_json")?,
                metadata_values_json: row.try_get("metadata_values_json")?,
                fail_message: row.try_get("fail_message")?,
            });
        }
        Ok(autoscaling_history)
    }
    pub async fn generate_autoscaling_history_samples(&self, sample_size: usize) -> Result<()> {
        for _ in 0..sample_size {
            let autoscaling_history = AutoscalingHistoryDefinition {
                id: Ulid::new().to_string(),
                plan_db_id: Ulid::new().to_string(),
                plan_id: Ulid::new().to_string(),
                plan_item_json: json!({
                    "id": Ulid::new().to_string(),
                    "item": rand::random::<f64>(),
                })
                .to_string(),
                metric_values_json: json!({
                    "id": Ulid::new().to_string(),
                    "value": rand::random::<f64>(),
                })
                .to_string(),
                metadata_values_json: json!({
                    "id": Ulid::new().to_string(),
                })
                .to_string(),
                fail_message: if rand::random() {
                    Some("test_fail_message".to_string())
                } else {
                    None
                },
            };
            self.add_autoscaling_history(autoscaling_history).await?;
        }
        Ok(())
    }
    // Remove the old AutoscalingHistory from the database
    pub async fn remove_old_autoscaling_history(&self, to_date: DateTime<Utc>) -> Result<()> {
        // e.g. 2021-01-01 00:00:00.000 + 0.001 -> 01F8ZQZ1Z0Z000000000000000
        let to_date = to_date + chrono::Duration::milliseconds(1);
        let to = Ulid::from_parts(to_date.timestamp_millis() as u64, 0).to_string();

        // Query
        let query_string = "DELETE FROM autoscaling_history WHERE id <= $1";
        let result = sqlx::query(query_string).bind(to).execute(&self.pool).await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::DataLayer;
    use super::*;
    use crate::data_layer::tests::{get_data_layer_with_postgres, get_data_layer_with_sqlite};
    use tracing::{debug, error};
    use tracing_test::traced_test;
    use ulid::Ulid;

    fn get_autoscaling_history_definition() -> AutoscalingHistoryDefinition {
        let id = Ulid::new().to_string();
        let plan_id = Ulid::new().to_string();
        let plan_db_id = Ulid::new().to_string();
        AutoscalingHistoryDefinition {
            id,
            plan_db_id,
            plan_id,
            plan_item_json: "test_plan_item_json".to_string(),
            metric_values_json: "test_metric_values_json".to_string(),
            metadata_values_json: "test_metadata_values_json".to_string(),
            fail_message: Some("test_fail_message".to_string()),
        }
    }

    #[tokio::test]
    #[traced_test]
    async fn test_autoscaling_history() {
        let data_layer = get_data_layer_with_sqlite().await;
        test_autoscaling_history_with_data_layer(data_layer).await;

        let data_layer = get_data_layer_with_postgres().await;
        test_autoscaling_history_with_data_layer(data_layer).await;
    }

    async fn test_autoscaling_history_with_data_layer(data_layer: DataLayer) {
        // Add a AutoscalingHistory to the database
        let autoscaling_history_definition = get_autoscaling_history_definition();
        error!(
            "autoscaling_history_definition: {:?}",
            autoscaling_history_definition
        );
        let result = data_layer
            .add_autoscaling_history(autoscaling_history_definition.clone())
            .await;
        debug!("result: {:?}", result);
        assert!(result.is_ok());

        // Get a AutoscalingHistory from the database
        let from_date = chrono::Utc::now() - chrono::Duration::days(1);
        let to_date = chrono::Utc::now();
        // let to_date = chrono::Utc::now();
        let result = data_layer
            .get_autoscaling_history_by_date(from_date, to_date)
            .await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(
            result[0].plan_db_id,
            autoscaling_history_definition.plan_db_id
        );

        // Get a AutoscalingHistory from the database by plan_id
        let result = data_layer
            .get_autoscaling_history_by_plan_id(autoscaling_history_definition.plan_id.clone())
            .await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result[0].plan_id, autoscaling_history_definition.plan_id);

        // Remove the old AutoscalingHistory from the database
        let result = data_layer.remove_old_autoscaling_history(to_date).await;
        assert!(result.is_ok());
        let result = data_layer
            .get_autoscaling_history_by_date(from_date, to_date)
            .await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 0);
    }
}
