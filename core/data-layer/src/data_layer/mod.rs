mod metric;
mod metrics_data;
mod plan_logs;
mod scaling_component;
mod scaling_plan;

use crate::types::metrics_data_item::MetricsDataItem;
use anyhow::{anyhow, Result};
use chrono::Utc;
use once_cell::sync::Lazy;
use serde_json::json;
use sqlx::{
    any::{AnyKind, AnyPoolOptions},
    AnyPool, Row,
};
use std::{collections::HashMap, path::Path};
use std::{
    collections::{BTreeMap, LinkedList},
    fs::File,
};
use std::{
    io::Read,
    sync::{Arc, RwLock},
};
use tokio::sync::watch;
use tracing::{debug, error, info};

const DEFAULT_DB_URL: &str = "sqlite://wave.db";
const DEFAULT_METRICS_DATA_BUFFER_SIZE_KB: u64 = 500_000;

/**
**MetricsData is a struct to store metrics data**
 */
#[derive(Debug)]
pub struct MetricsData {
    metric_buffer_size_byte: u64,
    enable_metrics_log: bool,
    pub metrics_data_map: HashMap<String, BTreeMap<String, MetricsDataItem>>,
    metrics_data_metadata: LinkedList<(String, String, usize)>,
    metrics_data_total_size: usize,
}

type SharedMetricsData = Arc<RwLock<MetricsData>>;

pub static METRICS_DATA: Lazy<SharedMetricsData> = Lazy::new(|| {
    let metrics_data = MetricsData {
        metric_buffer_size_byte: 500_000,
        enable_metrics_log: false,
        metrics_data_map: HashMap::new(),
        metrics_data_metadata: LinkedList::new(),
        metrics_data_total_size: 0,
    };
    Arc::new(RwLock::new(metrics_data))
});

#[derive(Debug)]
pub struct DataLayer {
    // Pool is a connection pool to the database. Postgres, Mysql, SQLite supported.
    pool: AnyPool,
    metrics_data: SharedMetricsData,
    action_sender: tokio::sync::broadcast::Sender<serde_json::Value>,
}

impl DataLayer {
    pub async fn new(sql_url: &str, metric_buffer_size_kb: u64, enable_metrics_log: bool) -> Self {
        let sql_url = if sql_url.is_empty() {
            DEFAULT_DB_URL
        } else {
            sql_url
        };
        let metric_buffer_size_byte = if metric_buffer_size_kb == 0 {
            DEFAULT_METRICS_DATA_BUFFER_SIZE_KB * 1000
        } else {
            metric_buffer_size_kb * 1000
        };

        {
            let metrics_data = METRICS_DATA.clone();
            let Ok(mut metrics_data) = metrics_data.write() else {
                error!("[DataLayer::new()] Failed to get the lock of metrics_data");
                panic!("Failed to get the lock of metrics_data");
            };
            metrics_data.metric_buffer_size_byte = metric_buffer_size_byte;
            metrics_data.enable_metrics_log = enable_metrics_log;
        }
        let (action_sender, _) = tokio::sync::broadcast::channel::<serde_json::Value>(16);

        DataLayer {
            pool: DataLayer::get_pool(sql_url).await,
            metrics_data: METRICS_DATA.clone(),
            action_sender,
        }
    }

    pub async fn sync(&self, definition_path: &str) {
        self.migrate().await;

        // TODO: Validate the definition file before loading it into the database
        let is_empty = definition_path.is_empty();
        let exists = Path::new(definition_path).exists();
        if !is_empty && exists {
            let result = self
                .load_definition_file_into_database(definition_path)
                .await;
            if result.is_err() {
                error!(
                    "Failed to load the definition file into the database: {:?}",
                    result
                );
            } else {
                info!(
                    "[data-layer] The definition file is loaded into the database: {}",
                    definition_path
                );
            }
        } else if !is_empty && !exists {
            error!("The definition file does not exist: {}", definition_path);
        } else {
            info!(
                "[data-layer] The definition path is empty, skip loading the definition file into the database"
            );
        }
    }

    async fn load_definition_file_into_database(&self, definition_path: &str) -> Result<()> {
        debug!("Loading the definition file into the database");

        // Read the file of the path
        let mut file = File::open(definition_path)?;
        let mut file_string = String::new();
        file.read_to_string(&mut file_string)?;

        // Parse the plan_file
        self.add_definitions(file_string.as_str()).await
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
                std::fs::File::create(path).unwrap();
            }
        }

        debug!("Connecting to the database: {}", sql_url);

        AnyPoolOptions::new()
            .max_connections(5)
            .connect(sql_url)
            .await
            .unwrap()
    }
    async fn migrate(&self) {
        debug!("Migrate to database type: {:?}", self.pool.any_kind());

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
                // Return error because MySQL is not supported yet
                panic!("MySQL is not supported yet");
            }
        }
    }
    pub fn watch_definitions_in_db(&self, watch_duration_ms: u64) -> watch::Receiver<String> {
        let (notify_sender, notify_receiver) = watch::channel(String::new());
        let pool = self.pool.clone();
        let database_kind = self.pool.any_kind();

        tokio::spawn(async move {
            let mut lastest_updated_at_hash: String = String::new();
            loop {
                // 1 second
                tokio::time::sleep(tokio::time::Duration::from_millis(watch_duration_ms)).await;
                debug!("Watching the definition in the db");

                // REFACTOR: Use type state pattern to avoid this match
                // watch all data for changed definition
                let query_string = match database_kind {
                    AnyKind::Postgres => {
                        "(SELECT updated_at FROM metric) UNION (SELECT updated_at FROM scaling_component) UNION (SELECT updated_at FROM plan)"
                    }
                    AnyKind::Sqlite => {
                        "SELECT updated_at FROM metric; SELECT updated_at FROM scaling_component; SELECT updated_at FROM plan;"
                    }
                    AnyKind::MySql => {
                        // Return error because MySQL is not supported yet
                        panic!("MySQL is not supported yet");
                    }
                };
                let result = sqlx::query(query_string).fetch_all(&pool).await;
                let Ok(result_string) = result else {
                    error!("Failed to fetch updated_at from the database, result: {:?}", result.err().unwrap());
                    continue;
                };
                let mut updated_at_hash_string: String = String::new();
                for row in &result_string {
                    if let Ok(updated_at) = row.try_get::<String, _>(0) {
                        updated_at_hash_string.push_str(&updated_at);
                    }
                }

                if lastest_updated_at_hash != updated_at_hash_string {
                    let timestamp = Utc::now().to_rfc3339();
                    let result = notify_sender.send(timestamp);
                    if result.is_err() {
                        error!(
                            "Failed to send notify signal - timestamp: {}",
                            result.err().unwrap().to_string()
                        );
                    }
                    lastest_updated_at_hash = updated_at_hash_string;
                }
            }
        });
        notify_receiver
    }

    pub async fn add_definitions(&self, yaml_str: &str) -> Result<()> {
        debug!("Loading the definition string into the database");

        let metric_definitions_result = self.sync_metric_yaml_for_no_match_id(yaml_str).await;
        if metric_definitions_result.is_err() {
            return Err(anyhow!("Failed to save metric definitions into DataLayer"));
        }

        // Save definitions into DataLayer
        let scaling_component_definitions_result = self
            .sync_scaling_component_yaml_for_no_match_id(yaml_str)
            .await;
        if scaling_component_definitions_result.is_err() {
            return Err(anyhow!(
                "Failed to save scaling component definitions into DataLayer"
            ));
        }

        // Save definitions into DataLayer
        let plan_definitions_result = self.add_plan_yaml(yaml_str).await;
        if plan_definitions_result.is_err() {
            return Err(anyhow!("Failed to save plan definitions into DataLayer"));
        }

        Ok(())
    }

    // Send an action to the action queue
    pub fn send_action(&self, action: serde_json::Value) -> Result<()> {
        let result: &std::prelude::v1::Result<
            usize,
            tokio::sync::broadcast::error::SendError<serde_json::Value>,
        > = &self.action_sender.send(action);
        if result.is_err() {
            let error_message = result.as_ref().err().unwrap().to_string();
            return Err(anyhow!(error_message));
        }
        Ok(())
    }
    pub fn send_plan_action(&self, plan_id: String, plan_item_id: String) -> Result<()> {
        let action = json!({
            "plan_id": plan_id,
            "plan_item_id": plan_item_id,
        });
        self.send_action(action)
    }
    // Get an receiver of the action
    pub fn subscribe_action(&self) -> tokio::sync::broadcast::Receiver<serde_json::Value> {
        self.action_sender.subscribe()
    }
}

#[cfg(test)]
pub mod tests {
    use super::DataLayer;
    use tracing::error;

    const DEFAULT_METRICS_DATA_BUFFER_SIZE_KB: u64 = 500_000;
    const DEFAULT_ENABLE_METRICS_LOG: bool = false;

    pub async fn get_data_layer_with_sqlite() -> DataLayer {
        const DEFAULT_DB_URL: &str = "sqlite://tests/temp/test.db";
        // Delete the test db if it exists
        let path = std::path::Path::new(DEFAULT_DB_URL.trim_start_matches("sqlite://"));
        let remove_result = std::fs::remove_file(path);
        if remove_result.is_err() {
            error!("Error removing file: {:?}", remove_result);
        }
        let data_layer = DataLayer::new(
            DEFAULT_DB_URL,
            DEFAULT_METRICS_DATA_BUFFER_SIZE_KB,
            DEFAULT_ENABLE_METRICS_LOG,
        )
        .await;
        data_layer.sync("").await;
        data_layer
    }

    pub async fn get_data_layer_with_postgres() -> DataLayer {
        const DEFAULT_DB_URL: &str = "postgres://postgres:postgres@localhost:5432/postgres";
        let data_layer = DataLayer::new(
            DEFAULT_DB_URL,
            DEFAULT_METRICS_DATA_BUFFER_SIZE_KB,
            DEFAULT_ENABLE_METRICS_LOG,
        )
        .await;
        data_layer.sync("").await;
        data_layer
    }
}
