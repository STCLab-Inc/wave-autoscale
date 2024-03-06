use super::DataLayer;
use crate::types::plan_log_definition::PlanLogDefinition;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::Row;
use ulid::Ulid;

impl DataLayer {
    // Add plan log to the database
    pub async fn add_plan_logs(&self, plan_log: PlanLogDefinition) -> Result<()> {
        let query_string = "INSERT INTO plan_log (id, plan_db_id, plan_id, plan_item_json, metric_values_json, metadata_values_json, fail_message) VALUES ($1,$2,$3,$4,$5,$6,$7)";
        let id = Ulid::new().to_string();
        let result = sqlx::query(query_string)
            // INTO
            .bind(id)
            .bind(plan_log.plan_db_id)
            .bind(plan_log.plan_id)
            .bind(plan_log.plan_item_json)
            .bind(plan_log.metric_values_json)
            .bind(plan_log.metadata_values_json)
            .bind(plan_log.fail_message)
            .execute(&self.pool)
            .await;

        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        Ok(())
    }
    // Get plan log by from and to date from the database
    pub async fn get_plan_logs_by_date(
        &self,
        plan_id: Option<String>,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
    ) -> Result<Vec<PlanLogDefinition>> {
        let mut plan_logs: Vec<PlanLogDefinition> = Vec::new();
        // Convert from and to date to Ulid.
        // e.g. 2021-01-01 00:00:00.000 -> 01F8ZQZ1Z0Z000000000000000
        let from = Ulid::from_parts(from_date.timestamp_millis() as u64, 0).to_string();
        // e.g. 2021-01-01 00:00:00.000 + 0.001 -> 01F8ZQZ1Z0Z000000000000000
        let to_date = to_date + chrono::Duration::milliseconds(1);
        let to = Ulid::from_parts(to_date.timestamp_millis() as u64, 0).to_string();

        // Query
        let mut query_string = "SELECT id, plan_db_id, plan_id, plan_item_json, metric_values_json, metadata_values_json, fail_message FROM plan_log WHERE id BETWEEN $1 AND $2".to_string();
        if plan_id.is_some() {
            query_string += " AND plan_id=$3";
        }
        let mut query = sqlx::query(query_string.as_str()).bind(from).bind(to);

        if plan_id.is_some() {
            query = query.bind(plan_id.unwrap());
        }

        let result = query.fetch_all(&self.pool).await;

        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();

        for row in result {
            plan_logs.push(PlanLogDefinition {
                id: row.try_get("id")?,
                plan_db_id: row.try_get("plan_db_id")?,
                plan_id: row.try_get("plan_id")?,
                plan_item_json: row.try_get("plan_item_json")?,
                metric_values_json: row.try_get("metric_values_json")?,
                metadata_values_json: row.try_get("metadata_values_json")?,
                fail_message: row.try_get("fail_message")?,
            });
        }
        Ok(plan_logs)
    }
    pub async fn generate_plan_log_samples(&self, sample_size: usize) -> Result<()> {
        for _ in 0..sample_size {
            let plan_logs = PlanLogDefinition {
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
            self.add_plan_logs(plan_logs).await?;
        }
        Ok(())
    }
    // Remove the old plan logs from the database
    pub async fn remove_old_plan_logs_in_db(&self, to_date: DateTime<Utc>) -> Result<()> {
        // e.g. 2021-01-01 00:00:00.000 + 0.001 -> 01F8ZQZ1Z0Z000000000000000
        let to_date = to_date + chrono::Duration::milliseconds(1);
        let to = Ulid::from_parts(to_date.timestamp_millis() as u64, 0).to_string();

        // Query
        let query_string = "DELETE FROM plan_log WHERE id <= $1";
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
    use tracing::debug;
    use tracing_test::traced_test;
    use ulid::Ulid;

    fn get_plan_log_definition() -> PlanLogDefinition {
        let id = Ulid::new().to_string();
        let plan_id = Ulid::new().to_string();
        let plan_db_id = Ulid::new().to_string();
        PlanLogDefinition {
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
    async fn test_plan_log() {
        let data_layer = get_data_layer_with_sqlite().await;
        test_plan_log_with_data_layer(data_layer).await;

        let data_layer = get_data_layer_with_postgres().await;
        test_plan_log_with_data_layer(data_layer).await;
    }

    async fn test_plan_log_with_data_layer(data_layer: DataLayer) {
        // Add a plan log to the database
        let plan_log_definition = get_plan_log_definition();
        let result = data_layer.add_plan_logs(plan_log_definition.clone()).await;
        debug!("result: {:?}", result);
        assert!(result.is_ok());

        // Get a plan log from the database
        let from_date = chrono::Utc::now() - chrono::Duration::days(1);
        let to_date = chrono::Utc::now();
        // let to_date = chrono::Utc::now();
        let result = data_layer
            .get_plan_logs_by_date(None, from_date, to_date)
            .await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result[0].plan_db_id, plan_log_definition.plan_db_id);

        // Get a plan log from the database by plan_id
        let result = data_layer
            .get_plan_logs_by_date(
                Some(plan_log_definition.plan_id.clone()),
                from_date,
                to_date,
            )
            .await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result[0].plan_id, plan_log_definition.plan_id);

        // Remove the old plan log from the database
        let result = data_layer.remove_old_plan_logs_in_db(to_date).await;
        assert!(result.is_ok());
        let result = data_layer
            .get_plan_logs_by_date(None, from_date, to_date)
            .await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 0);
    }
}
