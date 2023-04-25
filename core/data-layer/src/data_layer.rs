use anyhow::{anyhow, Result};
use sqlx::{
    any::{AnyKind, AnyPoolOptions, AnyQueryResult},
    AnyPool, Row,
};
use std::path::Path;
use uuid::Uuid;

use crate::{types::object_kind::ObjectKind, MetricDefinition};

#[derive(Debug)]
pub struct DataLayer {
    // Pool is a connection pool to the database. Postgres, Mysql, SQLite supported.
    pool: AnyPool,
}

pub struct DataLayerNewParam {
    pub sql_url: String,
}
impl DataLayer {
    pub async fn new(params: DataLayerNewParam) -> Self {
        let data_layer = DataLayer {
            pool: DataLayer::get_pool(&params.sql_url).await,
        };
        data_layer.migrate().await;
        return data_layer;
    }
    async fn get_pool(sql_url: &str) -> AnyPool {
        const SQLITE_PROTOCOL: &str = "sqlite://";
        if sql_url.contains(SQLITE_PROTOCOL) {
            // Create the SQLite file and directories if they don't exist
            let path = Path::new(sql_url.trim_start_matches(SQLITE_PROTOCOL));
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            // Create the SQLite file if it doesn't exist
            if !path.exists() {
                std::fs::File::create(&path).unwrap();
            }
        }
        let pool = AnyPoolOptions::new()
            .max_connections(5)
            .connect(sql_url)
            .await
            .unwrap();
        return pool;
    }
    async fn migrate(&self) {
        match &self.pool.any_kind() {
            AnyKind::Postgres => {
                sqlx::migrate!("migrations/postgres")
                    .run(&self.pool)
                    .await
                    .unwrap();
            }
            AnyKind::Sqlite => {
                sqlx::migrate!("migrations/sqlite")
                    .run(&self.pool)
                    .await
                    .unwrap();
            }
            AnyKind::MySql => {
                sqlx::migrate!("migrations/mysql")
                    .run(&self.pool)
                    .await
                    .unwrap();
            }
        }
    }
    // Add multiple metrics to the database
    pub async fn add_metrics(&self, metrics: Vec<MetricDefinition>) -> Result<()> {
        // Define a pool variable that is a trait to pass to the execute function
        for metric in metrics {
            let metadata_string = serde_json::to_string(&metric.metadata).unwrap();
            let query_string =
                "INSERT INTO metric (db_id, id, metric_kind, metadata) VALUES (?,?,?,?)";
            let id = Uuid::new_v4().to_string();

            let result = sqlx::query(query_string)
                .bind(id)
                .bind(metric.id)
                .bind(metric.metric_kind)
                .bind(metadata_string)
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
        let query_string = "SELECT db_id, id, metric_kind, metadata FROM metric";
        let result = sqlx::query(query_string).fetch_all(&self.pool).await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();
        for row in result {
            metrics.push(MetricDefinition {
                kind: ObjectKind::Metric,
                db_id: row.get("db_id"),
                id: row.get("id"),
                metric_kind: row.get("metric_kind"),
                metadata: serde_json::from_str(row.get("metadata")).unwrap(),
            });
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
    // Delete a metric
    pub async fn delete_metric(&self, db_id: String) -> Result<AnyQueryResult> {
        let query_string = "DELETE FROM metric WHERE db_id=?";
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
    // Update a metric in the database
    pub async fn update_metric(&self, metric: MetricDefinition) -> Result<AnyQueryResult> {
        let metadata_string = serde_json::to_string(&metric.metadata).unwrap();
        let query_string = "UPDATE metric SET id=?, metric_kind=?, metadata=? WHERE db_id=?";
        let result = sqlx::query(query_string)
            .bind(metric.id)
            .bind(metric.metric_kind)
            .bind(metadata_string)
            .bind(metric.db_id)
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
