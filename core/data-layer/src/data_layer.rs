use crate::{
    reader::wave_definition_reader::read_definition_yaml,
    types::{
        autoscaling_history_definition::AutoscalingHistoryDefinition, object_kind::ObjectKind,
        source_metrics::SourceMetrics,
    },
    variable_mapper::{execute_variable_mapper, get_variable_mapper},
    MetricDefinition, ScalingComponentDefinition, ScalingPlanDefinition,
};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use get_size::GetSize;
use once_cell::sync::Lazy;
use serde_json::json;
use sqlx::{
    any::{AnyKind, AnyPoolOptions, AnyQueryResult},
    AnyPool, Row,
};
use std::{
    collections::HashMap,
    path::Path,
    time::{Duration, SystemTime},
};
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
use ulid::Ulid;
use uuid::Uuid;

const DEFAULT_DB_URL: &str = "sqlite://wave.db";
const DEFAULT_METRIC_BUFFER_SIZE_KB: u64 = 500_000;

#[derive(Debug)]
pub struct SourceMetricsData {
    metric_buffer_size_byte: u64,
    enable_metrics_log: bool,
    pub source_metrics: HashMap<String, BTreeMap<String, SourceMetrics>>,
    source_metrics_metadata: LinkedList<(String, String, usize)>,
    source_metrics_size: usize,
}

type SharedSourceMetricsData = Arc<RwLock<SourceMetricsData>>;

pub static SOURCE_METRICS_DATA: Lazy<SharedSourceMetricsData> = Lazy::new(|| {
    let source_metrics_data = SourceMetricsData {
        metric_buffer_size_byte: 500_000,
        enable_metrics_log: false,
        source_metrics: HashMap::new(),
        source_metrics_metadata: LinkedList::new(),
        source_metrics_size: 0,
    };
    Arc::new(RwLock::new(source_metrics_data))
});

#[derive(Debug)]
pub struct DataLayer {
    // Pool is a connection pool to the database. Postgres, Mysql, SQLite supported.
    pool: AnyPool,
    source_metrics_data: SharedSourceMetricsData,
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
            DEFAULT_METRIC_BUFFER_SIZE_KB * 1000
        } else {
            metric_buffer_size_kb * 1000
        };

        {
            let source_metrics_data = SOURCE_METRICS_DATA.clone();
            let Ok(mut source_metrics_data) = source_metrics_data.write() else {
                error!("[DataLayer::new()] Failed to get the lock of source_metrics_data");
                panic!("Failed to get the lock of source_metrics_data");
            };
            source_metrics_data.metric_buffer_size_byte = metric_buffer_size_byte;
            source_metrics_data.enable_metrics_log = enable_metrics_log;
        }
        let (action_sender, _) = tokio::sync::broadcast::channel::<serde_json::Value>(16);

        DataLayer {
            pool: DataLayer::get_pool(sql_url).await,
            source_metrics_data: SOURCE_METRICS_DATA.clone(),
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
                    let updated_at: chrono::DateTime<Utc> = row.get(0);
                    updated_at_hash_string.push_str(&updated_at.to_string());
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

        // Parse the plan_file
        let parser_result = read_definition_yaml(yaml_str);
        if parser_result.is_err() {
            return Err(anyhow!(
                "Failed to parse the definition string: {:?}",
                parser_result
            ));
        }
        let parser_result = parser_result.unwrap();

        // Save definitions into DataLayer
        let metric_definitions = parser_result.metric_definitions.clone();
        let metric_definitions_result = self.add_metrics(metric_definitions).await;
        if metric_definitions_result.is_err() {
            return Err(anyhow!("Failed to save metric definitions into DataLayer"));
        }

        // Save definitions into DataLayer
        let scaling_component_definitions = parser_result.scaling_component_definitions.clone();
        let scaling_component_definitions_result = self
            .add_scaling_components(scaling_component_definitions)
            .await;
        if scaling_component_definitions_result.is_err() {
            return Err(anyhow!(
                "Failed to save scaling component definitions into DataLayer"
            ));
        }

        // Save definitions into DataLayer
        let scaling_plan_definitions = parser_result.scaling_plan_definitions.clone();
        let scaling_plan_definitions_result = self.add_plans(scaling_plan_definitions).await;
        if scaling_plan_definitions_result.is_err() {
            return Err(anyhow!(
                "Failed to save scaling plan definitions into DataLayer"
            ));
        }
        Ok(())
    }

    // Add multiple metrics to the database
    pub async fn add_metrics(&self, metrics: Vec<MetricDefinition>) -> Result<()> {
        // Define a pool variable that is a trait to pass to the execute function
        for metric in metrics {
            let metadata_string = serde_json::to_string(&metric.metadata).unwrap();
            let query_string =
                "INSERT INTO metric (db_id, id, collector, metadata, enabled, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT (id) DO UPDATE SET (collector, metadata, enabled, updated_at) = ($8,$9,$10,$11)";
            let db_id = Uuid::new_v4().to_string();
            let updated_at = Utc::now();
            let result = sqlx::query(query_string)
                // Values for insert
                .bind(db_id)
                .bind(metric.id.to_lowercase())
                .bind(metric.collector.to_lowercase())
                .bind(metadata_string.clone())
                .bind(metric.enabled)
                .bind(updated_at)
                .bind(updated_at)
                // Values for update
                .bind(metric.collector.to_lowercase())
                .bind(metadata_string.clone())
                .bind(metric.enabled)
                .bind(updated_at)
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

        let variable_mapper_data = get_variable_mapper();

        for row in result {
            let metadata = match row.try_get::<Option<&str>, _>("metadata") {
                Ok(Some(metadata_str)) => {
                    execute_variable_mapper(metadata_str.to_string(), &variable_mapper_data)
                        .map_err(|e| anyhow!("Error in execute_variable_mapper: {}", e))?
                }
                Ok(None) => serde_json::Value::Null.to_string(),
                Err(e) => return Err(anyhow!("Error getting metadata: {}", e)),
            };

            metrics.push(MetricDefinition {
                kind: ObjectKind::Metric,
                db_id: row.get("db_id"),
                id: row.get("id"),
                collector: row.get("collector"),
                metadata: serde_json::from_str(metadata.as_str()).unwrap(),
                enabled: row.get("enabled"),
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
    // Get a metric from the database
    pub async fn get_metric_by_id(&self, db_id: String) -> Result<Option<MetricDefinition>> {
        let query_string =
            "SELECT db_id, id, collector, metadata, enabled FROM metric WHERE db_id=$1";
        let result = sqlx::query(query_string)
            .bind(db_id)
            // Do not use fetch_one because it expects exact one result. If not, it will return an error
            .fetch_all(&self.pool)
            .await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();
        if result.is_empty() {
            return Ok(None);
        }
        let result = result.get(0).map(|row| MetricDefinition {
            kind: ObjectKind::Metric,
            db_id: row.get("db_id"),
            id: row.get("id"),
            collector: row.get("collector"),
            metadata: serde_json::from_str(row.get("metadata")).unwrap(),
            enabled: row.get("enabled"),
        });
        Ok(result)
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
        let query_string = "DELETE FROM metric WHERE db_id=$1";
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
        let query_string =
            "UPDATE metric SET id=$1, collector=$2, metadata=$3, updated_at=$4, enabled=$5 WHERE db_id=$6";
        let updated_at = Utc::now();
        let result = sqlx::query(query_string)
            // SET
            .bind(metric.id)
            .bind(metric.collector)
            .bind(metadata_string)
            .bind(updated_at)
            .bind(metric.enabled)
            // WHERE
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
            let updated_at = Utc::now();
            let result = sqlx::query(query_string)
                // Values for insert
                .bind(id)
                .bind(scaling_component.id)
                .bind(scaling_component.component_kind)
                .bind(metadata_string.clone())
                .bind(scaling_component.enabled)
                .bind(updated_at)
                .bind(updated_at)
                // Values for update
                .bind(metadata_string.clone())
                .bind(scaling_component.enabled)
                .bind(updated_at)
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
            let metadata = match row.try_get::<Option<&str>, _>("metadata") {
                Ok(Some(metadata_str)) => {
                    execute_variable_mapper(metadata_str.to_string(), &variable_mapper_data)
                        .map_err(|e| anyhow!("Error in execute_variable_mapper: {}", e))?
                }
                Ok(None) => serde_json::Value::Null.to_string(),
                Err(e) => return Err(anyhow!("Error getting metadata: {}", e)),
            };

            scaling_components.push(ScalingComponentDefinition {
                kind: ObjectKind::ScalingComponent,
                db_id: row.get("db_id"),
                id: row.get("id"),
                component_kind: row.get("component_kind"),
                metadata: serde_json::from_str(metadata.as_str()).unwrap(),
                enabled: row.get("enabled"),
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
        let updated_at = Utc::now();
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
    // Add multiple plans to the database
    pub async fn add_plans(&self, plans: Vec<ScalingPlanDefinition>) -> Result<()> {
        // Define a pool variable that is a trait to pass to the execute function
        for plan in plans {
            let plans_string = serde_json::to_string(&plan.plans).unwrap();
            let metatdata_string = serde_json::to_string(&plan.metadata).unwrap();
            let query_string = "INSERT INTO plan (db_id, id, metadata, plans, enabled, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT (id) DO UPDATE SET (metadata, plans, enabled, updated_at) = ($8, $9, $10, $11)";
            let id = Uuid::new_v4().to_string();
            let updated_at = Utc::now();
            let result = sqlx::query(query_string)
                // Values for insert
                .bind(id)
                .bind(plan.id)
                .bind(metatdata_string.clone())
                .bind(plans_string.clone())
                .bind(plan.enabled)
                .bind(updated_at)
                .bind(updated_at)
                // Values for update
                .bind(metatdata_string.clone())
                .bind(plans_string.clone())
                .bind(plan.enabled)
                .bind(updated_at)
                .execute(&self.pool)
                .await;
            if result.is_err() {
                return Err(anyhow!(result.err().unwrap().to_string()));
            }
        }
        Ok(())
    }
    // Get all plans from the database
    pub async fn get_all_plans(&self) -> Result<Vec<ScalingPlanDefinition>> {
        let mut plans: Vec<ScalingPlanDefinition> = Vec::new();
        let query_string = "SELECT db_id, id, plans, priority, metadata, enabled FROM plan";
        let result = sqlx::query(query_string).fetch_all(&self.pool).await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();

        let variable_mapper_data = get_variable_mapper();

        for row in result {
            let metadata = match row.try_get::<Option<&str>, _>("metadata") {
                Ok(Some(metadata_str)) => {
                    execute_variable_mapper(metadata_str.to_string(), &variable_mapper_data)
                        .map_err(|e| anyhow!("Error in execute_variable_mapper: {}", e))?
                }
                Ok(None) => serde_json::Value::Null.to_string(),
                Err(e) => return Err(anyhow!("Error getting metadata: {}", e)),
            };

            plans.push(ScalingPlanDefinition {
                kind: ObjectKind::ScalingPlan,
                db_id: row.get("db_id"),
                id: row.get("id"),
                metadata: serde_json::from_str(metadata.as_str()).unwrap(),
                plans: serde_json::from_str(row.get("plans")).unwrap(),
                enabled: row.get("enabled"),
            });
        }
        Ok(plans)
    }
    // Get enabled plans
    pub async fn get_enabled_plans(&self) -> Result<Vec<ScalingPlanDefinition>> {
        let plans = self.get_all_plans().await?;
        let plans = plans
            .into_iter()
            .filter(|plan| plan.enabled)
            .collect::<Vec<ScalingPlanDefinition>>();
        Ok(plans)
    }
    // Get all plans json from the database
    pub async fn get_all_plans_json(&self) -> Result<Vec<serde_json::Value>> {
        let mut plans: Vec<serde_json::Value> = Vec::new();
        let query_string =
            "SELECT db_id, id, plans, priority, metadata, enabled, created_at, updated_at FROM plan";
        let result = sqlx::query(query_string).fetch_all(&self.pool).await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();
        for row in result {
            let plan = json!({
                "kind": ObjectKind::ScalingPlan,
                "db_id": row.try_get::<String, _>("db_id")?,
                "id": row.try_get::<String, _>("id")?,
                "plans": serde_json::from_str::<serde_json::Value>(row.try_get::<String, _>("plans")?.as_str())?,
                "metadata": serde_json::from_str::<serde_json::Value>(row.try_get::<String, _>("metadata")?.as_str())?,
                "enabled": row.try_get::<bool, _>("enabled")?,
                "created_at": row.try_get::<Option<String>, _>("created_at")?,
                "updated_at": row.try_get::<Option<String>, _>("updated_at")?,
            });
            plans.push(plan);
        }
        Ok(plans)
    }
    // Get a plan from the database
    pub async fn get_plan_by_id(&self, db_id: String) -> Result<ScalingPlanDefinition> {
        let query_string = "SELECT db_id, id, metadata, plans, enabled FROM plan WHERE db_id=$1";
        let result = sqlx::query(query_string)
            .bind(db_id)
            .fetch_one(&self.pool)
            .await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();
        let plan = ScalingPlanDefinition {
            kind: ObjectKind::ScalingPlan,
            db_id: result.get("db_id"),
            id: result.get("id"),
            metadata: serde_json::from_str(result.get("metadata")).unwrap(),
            plans: serde_json::from_str(result.get("plans")).unwrap(),
            enabled: result.get("enabled"),
        };
        Ok(plan)
    }
    // Delete all plans from the database
    pub async fn delete_all_plans(&self) -> Result<()> {
        let query_string = "DELETE FROM plan";
        let result = sqlx::query(query_string).execute(&self.pool).await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        Ok(())
    }
    // Delete a plan
    pub async fn delete_plan(&self, db_id: String) -> Result<AnyQueryResult> {
        let query_string = "DELETE FROM plan WHERE db_id=$1";
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
    // Update a plan in the database
    pub async fn update_plan(&self, plan: ScalingPlanDefinition) -> Result<AnyQueryResult> {
        let plans_string = serde_json::to_string(&plan.plans).unwrap();
        let metatdata_string = serde_json::to_string(&plan.metadata).unwrap();
        let query_string =
            "UPDATE plan SET id=$1, metadata=$2, plans=$3, updated_at=$4, enabled=$5 WHERE db_id=$6";
        let updated_at = Utc::now();
        let result = sqlx::query(query_string)
            // SET
            .bind(plan.id)
            .bind(metatdata_string)
            .bind(plans_string)
            .bind(updated_at)
            .bind(plan.enabled)
            // WHERE
            .bind(plan.db_id)
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
                id: row.get("id"),
                plan_db_id: row.get("plan_db_id"),
                plan_id: row.get("plan_id"),
                plan_item_json: row.get("plan_item_json"),
                metric_values_json: row.get("metric_values_json"),
                metadata_values_json: row.get("metadata_values_json"),
                fail_message: row.get("fail_message"),
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
                id: row.get("id"),
                plan_db_id: row.get("plan_db_id"),
                plan_id: row.get("plan_id"),
                plan_item_json: row.get("plan_item_json"),
                metric_values_json: row.get("metric_values_json"),
                metadata_values_json: row.get("metadata_values_json"),
                fail_message: row.get("fail_message"),
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

    // Source Metrics
    pub async fn add_source_metrics_in_data_layer(
        &self,
        collector: &str,
        metric_id: &str,
        json_value: &str,
    ) -> Result<()> {
        /* [ Comment ]
         *  source metrics: Metric data is separated by metric_id and ulid is sorted in ascending order (using for ScalingPlan search)
         *  source metrics metadata: Metric data is sorted in ASC order by ULID (using for remove target data to maintain buffer size)
         * [ Data structure ]
         *  source metrics - HashMap<key: metric_id, value: BTreeMap<key: ULID, value: SourceMetrics>>
         *  source metrics metadata - LinkedList<(metric_id, ULID, data size(source metrics + source metrics metadata)> - list order by ULID ASC */
        let Ok(mut source_metrics_data) = self.source_metrics_data.write() else {
            error!("[add_source_metrics_in_data_layer] Failed to get source metrics data");
            return Err(anyhow!("Failed to get source metrics data"));
        };

        let now_ulid = Ulid::new().to_string();
        let source_metric_insert_data = SourceMetrics {
            json_value: json_value.to_string(),
        };
        // source_metric_insert_data_size = source metrics + source metrics metadata
        let source_metric_insert_data_size = source_metric_insert_data.get_heap_size()
            + (metric_id.to_string().get_heap_size() * 2)
            + (now_ulid.get_heap_size() * 2);

        let mut total_source_metrics_size =
            source_metrics_data.source_metrics_size + source_metric_insert_data_size;

        // get remove target data
        let mut remove_target_data: Vec<(String, String, usize)> = Vec::new();
        let mut subtract_total_size = total_source_metrics_size;
        loop {
            // check size :: buffersize - total size < 0 (None) => remove target data
            if source_metrics_data
                .metric_buffer_size_byte
                .checked_sub(subtract_total_size as u64)
                .is_some()
            {
                break;
            }
            let Some(front_source_metrics_metadata) = source_metrics_data.source_metrics_metadata.pop_front() else {
                break;
            };
            subtract_total_size -= front_source_metrics_metadata.2; // oldest metadata size
            remove_target_data.append(&mut vec![front_source_metrics_metadata]);
        }

        // remove target data
        remove_target_data
            .iter()
            .for_each(|(metric_id, ulid, size)| {
                let Some(source_metrics_map) = source_metrics_data.source_metrics.get_mut(metric_id) else {
                    return;
                };
                // remove source metrics - btreemap ulid
                source_metrics_map.remove(ulid);
                // (if btreemap is empty) remove source metrics - hashmap metric_id
                if source_metrics_map.is_empty() {
                    source_metrics_data.source_metrics.remove(metric_id);
                }
                total_source_metrics_size -= size;
            });

        // add source metrics
        match source_metrics_data.source_metrics.get_mut(metric_id) {
            Some(source_metrics_map) => {
                source_metrics_map.insert(now_ulid.clone(), source_metric_insert_data.clone());
            }
            None => {
                let mut source_metrics_map = BTreeMap::new();
                source_metrics_map.insert(now_ulid.clone(), source_metric_insert_data.clone());
                source_metrics_data
                    .source_metrics
                    .insert(metric_id.to_string(), source_metrics_map);
            }
        }
        // add source metrics metadata
        source_metrics_data.source_metrics_metadata.push_back((
            metric_id.to_string(),
            now_ulid,
            source_metric_insert_data_size,
        ));

        debug!(
            "[source metrics] add item size: {} / total size: {}",
            source_metric_insert_data_size, total_source_metrics_size
        );

        // update source_metric_size
        source_metrics_data.source_metrics_size = total_source_metrics_size;

        // debug!("Save Metric\n{:?}", source_metric_insert_data);

        // Save to database
        if source_metrics_data.enable_metrics_log {
            let result_save_db = self
                .add_source_metric(collector, metric_id, json_value)
                .await;
            if result_save_db.is_err() {
                error!("Failed to save metric into the DB: {:?}", result_save_db);
            }
        }
        Ok(())
    }

    // Add a SourceMetric to the database
    pub async fn add_source_metric(
        &self,
        collector: &str,
        metric_id: &str,
        json_value: &str,
    ) -> Result<()> {
        // TODO: Validate json_value
        // json_value has to follow the below format('tags' is optional)
        // {
        //     "name": "cpu_usage",
        //     "tags": {
        //        "host": "localhost",
        //        "region": "us-west"
        //     },
        //    "value": 0.64,
        // }
        //
        let query_string =
            "INSERT INTO source_metrics (id, collector, metric_id, json_value) VALUES ($1,$2,$3,$4)";
        // ULID as id instead of UUID because of the time based sorting
        let id = Ulid::new().to_string();
        let result = sqlx::query(query_string)
            // VALUE
            .bind(id)
            .bind(collector)
            .bind(metric_id)
            .bind(json_value)
            .execute(&self.pool)
            .await;
        debug!("result: {:?}", result);
        if result.is_err() {
            let error_message = result.err().unwrap().to_string();
            error!("Error: {}", error_message);
            return Err(anyhow!(error_message));
        }
        debug!("Added a source metric: {}", metric_id);
        Ok(())
    }
    // Get a latest metrics by collector and metric_id from the database
    pub async fn get_source_metrics_values(
        &self,
        metric_ids: Vec<String>,
        time_greater_than: u64,
    ) -> Result<HashMap<String, serde_json::Value>> {
        // Subtract the duration from the current time
        //
        let offset_time = SystemTime::now() - Duration::from_millis(time_greater_than);
        let ulid = Ulid::from_datetime(offset_time);
        let query_string =
            "SELECT metric_id, id, json_value FROM source_metrics WHERE id >= $1 and metric_id in ($2) ORDER BY id DESC LIMIT 1";
        let metric_ids = metric_ids.join(",");
        let result = sqlx::query(query_string)
            .bind(ulid.to_string())
            .bind(metric_ids)
            .fetch_all(&self.pool)
            .await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();
        let mut metric_values: HashMap<String, serde_json::Value> = HashMap::new();
        for row in result {
            let metric_id: String = row.get("metric_id");
            let json_value: String = row.get("json_value");
            let json_value = json!(json_value);
            metric_values.insert(metric_id, json_value);
        }
        Ok(metric_values)
    }

    // Get a latest metrics by collector from the database
    pub async fn get_source_metrics_values_all_metric_ids(
        &self,
        read_before_ms: u64,
    ) -> Result<Vec<serde_json::Value>> {
        let offset_time = SystemTime::now() - Duration::from_millis(read_before_ms);
        let ulid = Ulid::from_datetime(offset_time);
        let query_string = "SELECT metric_id, id, json_value FROM source_metrics WHERE id >= $1";
        let result = sqlx::query(query_string)
            .bind(ulid.to_string())
            .fetch_all(&self.pool)
            .await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();
        let mut metric_values: Vec<serde_json::Value> = Vec::new();
        for row in result {
            let metric_id: String = row.get("metric_id");
            let id: String = row.get("id");
            let json_value: String = row.get("json_value");
            let json_value = json!({"metric_id": metric_id, "id": id, "json_value": json_value});
            metric_values.append(&mut vec![json_value]);
        }
        Ok(metric_values)
    }

    // Get inflow metric id
    pub async fn get_inflow_metric_id(&self) -> Result<Vec<String>> {
        let mut metric_ids: Vec<String> = Vec::new();

        // Acquire read lock on source metrics data
        let source_metrics_data = match SOURCE_METRICS_DATA.read() {
            Ok(data) => data,
            Err(err) => {
                eprintln!(
                    "Failed to acquire read lock on SOURCE_METRICS_DATA: {}",
                    err
                );
                return Ok(metric_ids);
            }
        };

        // Extract metric_ids from source_metrics
        for metric_id in source_metrics_data.source_metrics.keys() {
            metric_ids.push(metric_id.clone());
        }

        Ok(metric_ids)
    }
    // Get inflow with metric id by from and to date
    pub async fn get_inflow_with_metric_id_by_date(
        &self,
        metric_id: String,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
    ) -> Result<Vec<SourceMetrics>> {
        let mut inflow: Vec<SourceMetrics> = Vec::new();
        let from = from_date.timestamp_millis();
        let to = (to_date + chrono::Duration::milliseconds(1)).timestamp_millis();

        let mut source_metrics: Vec<SourceMetrics> = Vec::new();

        // Acquire read lock on source metrics data
        let source_metrics_data = match SOURCE_METRICS_DATA.read() {
            Ok(data) => data,
            Err(err) => {
                eprintln!(
                    "Failed to acquire read lock on SOURCE_METRICS_DATA: {}",
                    err
                );
                return Ok(inflow);
            }
        };

        // Extract source metrics data with metric id
        if let Some(source_metrics_item) = source_metrics_data.source_metrics.get(&metric_id) {
            // Extract json_value from the source_metrics_item
            for (_key, value) in source_metrics_item.iter() {
                source_metrics.push(SourceMetrics {
                    json_value: value.json_value.clone(),
                });
            }
        }

        // Iterate over source metrics data and filter based on timestamp
        for source_metrics_item in source_metrics.iter() {
            // Parse json_value from the entry
            let json_value: serde_json::Value =
                match serde_json::from_str(&source_metrics_item.json_value) {
                    Ok(value) => value,
                    Err(err) => {
                        eprintln!("Failed to parse JSON: {}", err);
                        continue;
                    }
                };

            // Check if json value is an array
            if let Some(json_values) = json_value.as_array() {
                // Iterate over each object in the array
                for json_values_item in json_values {
                    // Check if the json values item has a timestamp field
                    if let Some(json_values_item_timestamp) = json_values_item
                        .get("timestamp")
                        .and_then(|timestamp| timestamp.as_str())
                    {
                        if let Ok(parse_json_values_item_timestamp) =
                            DateTime::parse_from_str(json_values_item_timestamp, "%+")
                                .map(|datetime| datetime.with_timezone(&Utc))
                        {
                            // Check if the timestamp is within the specified range
                            if parse_json_values_item_timestamp.timestamp_millis() >= from
                                && parse_json_values_item_timestamp.timestamp_millis() <= to
                            {
                                // Add to the result vector
                                inflow.push(SourceMetrics {
                                    json_value: json_values_item.to_string(),
                                });
                                // Break out of the loop if at least one valid entry is found
                                break;
                            }
                        } else {
                            eprintln!(
                                "Failed to parse timestamp from string: {}",
                                json_values_item_timestamp
                            );
                        }
                    } else {
                        // Print a message indicating that the timestamp field is not found
                        eprintln!(
                            "Timestamp field is not found in array element: {:?}",
                            json_values_item
                        );
                    }
                }
            } else {
                eprintln!("Failed to read from array: {:?}", json_value);
            }
        }

        Ok(inflow)
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
mod tests {
    use super::DataLayer;
    use super::*;
    use crate::types::autoscaling_history_definition::AutoscalingHistoryDefinition;
    use tracing::{debug, error};
    use tracing_test::traced_test;
    use ulid::Ulid;
    const DEFAULT_METRIC_BUFFER_SIZE_KB: u64 = 500_000;
    const DEFAULT_ENABLE_METRICS_LOG: bool = false;

    async fn get_data_layer_with_sqlite() -> DataLayer {
        const DEFAULT_DB_URL: &str = "sqlite://tests/temp/test.db";
        // Delete the test db if it exists
        let path = std::path::Path::new(DEFAULT_DB_URL.trim_start_matches("sqlite://"));
        let remove_result = std::fs::remove_file(path);
        if remove_result.is_err() {
            error!("Error removing file: {:?}", remove_result);
        }
        let data_layer = DataLayer::new(
            DEFAULT_DB_URL,
            DEFAULT_METRIC_BUFFER_SIZE_KB,
            DEFAULT_ENABLE_METRICS_LOG,
        )
        .await;
        data_layer.sync("").await;
        data_layer
    }

    async fn get_data_layer_with_postgres() -> DataLayer {
        const DEFAULT_DB_URL: &str = "postgres://postgres:postgres@localhost:5432/postgres";
        let data_layer = DataLayer::new(
            DEFAULT_DB_URL,
            DEFAULT_METRIC_BUFFER_SIZE_KB,
            DEFAULT_ENABLE_METRICS_LOG,
        )
        .await;
        data_layer.sync("").await;
        data_layer
    }

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

    #[tokio::test]
    #[traced_test]
    async fn test_get_source_metrics_values_all_metric_ids() {
        let data_layer = get_data_layer_with_sqlite().await;
        test_get_source_metrics_values_all_metric_ids_with_data_layer(data_layer).await;

        let data_layer = get_data_layer_with_postgres().await;
        test_get_source_metrics_values_all_metric_ids_with_data_layer(data_layer).await;
    }
    async fn test_get_source_metrics_values_all_metric_ids_with_data_layer(data_layer: DataLayer) {
        let json_value = r#"[{
            "name": "test",
            "tags": {
                "tag1": "value1"
            },
            "value": 2.0
        }]
        "#;

        // add a source metric
        let add_source_metric = data_layer
            .add_source_metric("vector", "source_metrics_test_1", json_value)
            .await;
        assert!(add_source_metric.is_ok());

        // read source metric
        let source_metrics = data_layer
            .get_source_metrics_values_all_metric_ids(10 * 1000)
            .await
            .unwrap();
        let source_metrics_filter_arr: Vec<&serde_json::Value> = source_metrics
            .iter()
            .filter(|value| value.get("metric_id").unwrap() == "source_metrics_test_1")
            .collect();
        assert!(!source_metrics_filter_arr.is_empty());
    }

    #[tokio::test]
    async fn test_add_source_metrics_in_data_layer() {
        const DB_URL: &str = "sqlite://tests/temp/test.db";
        const METRIC_BUFFER_SIZE_KB: u64 = 1;
        let data_layer =
            DataLayer::new(DB_URL, METRIC_BUFFER_SIZE_KB, DEFAULT_ENABLE_METRICS_LOG).await;
        let collector = "vector";
        let metric_id = "metric_1";
        let json_value = r#"[{
            "name": "name",
            "tags": {
                "tag1": "value1"
            },
            "value": 2.0
        }]"#;
        for _idx in 1..10 {
            let _ = data_layer
                .add_source_metrics_in_data_layer(collector, metric_id, json_value)
                .await;
        }
        let mut total_source_metric_size = 0;
        SOURCE_METRICS_DATA
            .read()
            .unwrap()
            .source_metrics
            .iter()
            .for_each(|source_metric| {
                source_metric.1.iter().for_each(|(ulid, source_metrics)| {
                    total_source_metric_size += source_metrics.get_heap_size()
                        + (metric_id.get_heap_size() * 2)
                        + (ulid.get_heap_size() * 2);
                });
            });

        let measure_json_value_size = SourceMetrics {
            json_value: json_value.to_string(),
        }
        .get_heap_size();
        let measure_metric_id_size = metric_id.get_heap_size() * 2;
        let measure_ulid_size = Ulid::new().to_string().get_heap_size() * 2;
        let measure_size = measure_json_value_size + measure_metric_id_size + measure_ulid_size;
        assert!(total_source_metric_size < (METRIC_BUFFER_SIZE_KB * 1000) as usize); // source_metrics of size is less than METRIC_BUFFER_SIZE_KB
        assert!(total_source_metric_size % measure_size == 0); // source_metrics of size is multiple of measure_size
    }

    async fn add_source_metrics_in_data_layer_save_test_data(
        data_layer: &DataLayer,
        metric_id: String,
        json_value: String,
        ulid_size: usize,
    ) -> usize {
        // sample data 1
        let metric_id_size = metric_id.get_heap_size();
        let json_value_data = SourceMetrics {
            json_value: json_value.to_string(),
        };
        let sample_data_total_size =
            json_value_data.get_heap_size() + (metric_id_size * 2) + (ulid_size * 2);

        // save sample data 1
        let _ = data_layer
            .add_source_metrics_in_data_layer("vector", &metric_id, &json_value)
            .await;
        sample_data_total_size
    }

    async fn get_add_source_metrics_in_data_layer_size(_data_layer: &DataLayer) -> usize {
        let mut total_source_metrics_size = 0;
        for (metric_id, btree_map) in SOURCE_METRICS_DATA.read().unwrap().source_metrics.iter() {
            for (ulid, source_metrics) in btree_map.iter() {
                let source_metrics_size = source_metrics.get_heap_size()
                    + (metric_id.get_heap_size() * 2)
                    + (ulid.get_heap_size() * 2);
                total_source_metrics_size += source_metrics_size;
            }
        }
        total_source_metrics_size
    }

    async fn get_add_source_metrics_metadata_in_data_layer_size(_data_layer: &DataLayer) -> usize {
        let mut total_source_metrics_metadata_size = 0;
        SOURCE_METRICS_DATA
            .read()
            .unwrap()
            .source_metrics_metadata
            .iter()
            .for_each(|(_metric_id, _ulid, size)| {
                total_source_metrics_metadata_size += size;
            });
        total_source_metrics_metadata_size
    }

    #[tokio::test]
    async fn test_add_source_metrics_in_data_layer_check_save_data() {
        const DB_URL: &str = "sqlite://tests/temp/test.db";
        const METRIC_BUFFER_SIZE_KB: u64 = 1;
        let data_layer =
            DataLayer::new(DB_URL, METRIC_BUFFER_SIZE_KB, DEFAULT_ENABLE_METRICS_LOG).await;
        let ulid_size = Ulid::new().to_string().get_heap_size();

        // sample data 1
        let sample_1_metric_id = "sample_1".to_string();
        let sample_1_json_value =
            json!([{"name": "test", "tags": {"tag1": "value","tag2": "value","tag3": "value"}, "value": 1.0}]).to_string();
        let sample_1_total_size = add_source_metrics_in_data_layer_save_test_data(
            &data_layer,
            sample_1_metric_id,
            sample_1_json_value,
            ulid_size,
        )
        .await;

        // sample data 2
        let sample_2_metric_id = "sample_2".to_string();
        let sample_2_json_value =
            json!([{"name": "test", "tags": {"tag1": "value","tag2": "value","tag3": "value","tag4": "value","tag5": "value","tag6": "value","tag7": "value","tag8": "value","tag9": "value","tag10": "value","tag11": "value","tag12": "value","tag13": "value","tag14": "value","tag15": "value","tag16": "value","tag17": "value","tag18": "value","tag19": "value","tag20": "value","tag21": "value"}, "value": 1.0}]).to_string();
        let sample_2_total_size = add_source_metrics_in_data_layer_save_test_data(
            &data_layer,
            sample_2_metric_id,
            sample_2_json_value,
            ulid_size,
        )
        .await;

        // sample data 3
        let sample_3_metric_id = "sample_1".to_string();
        let sample_3_json_value =
            json!([{"name": "test", "tags": {"tag1": "value","tag2": "value","tag3": "value","tag4": "value","tag5": "value","tag6": "value","tag7": "value","tag8": "value","tag9": "value","tag10": "value"}, "value": 1.0}]).to_string();
        let sample_3_total_size = add_source_metrics_in_data_layer_save_test_data(
            &data_layer,
            sample_3_metric_id,
            sample_3_json_value,
            ulid_size,
        )
        .await;

        // check save data size = source_metrics
        let total_source_metrics_size =
            get_add_source_metrics_in_data_layer_size(&data_layer).await;

        // check save data size => source_metrics_metadata
        let total_source_metrics_metadata_size =
            get_add_source_metrics_metadata_in_data_layer_size(&data_layer).await;

        // sample data size(1 + 2 + 3) = total source metrics size
        assert!(
            (sample_1_total_size + sample_2_total_size + sample_3_total_size)
                == total_source_metrics_size
        );
        // sample data size(1 + 2 + 3) = total source metrics metadata size
        assert!(
            (sample_1_total_size + sample_2_total_size + sample_3_total_size)
                == total_source_metrics_metadata_size
        );

        // sample data 4
        let sample_4_metric_id = "sample_4".to_string();
        let sample_4_json_value =
            json!([{"name": "test", "tags": {"tag1": "value","tag2": "value","tag3": "value","tag4": "value","tag5": "value","tag6": "value","tag7": "value","tag8": "value","tag9": "value","tag10": "value","tag11": "value","tag12": "value","tag13": "value","tag14": "value","tag15": "value"}, "value": 1.0}]).to_string();
        let sample_4_total_size = add_source_metrics_in_data_layer_save_test_data(
            &data_layer,
            sample_4_metric_id,
            sample_4_json_value,
            ulid_size,
        )
        .await;
        // check save data size = source_metrics
        let total_source_metrics_size_2 =
            get_add_source_metrics_in_data_layer_size(&data_layer).await;

        // check save data size => source_metrics_metadata
        let total_source_metrics_metadata_size_2 =
            get_add_source_metrics_metadata_in_data_layer_size(&data_layer).await;

        // sample data size(3 + 4) = total source metrics size
        assert!((sample_3_total_size + sample_4_total_size) == total_source_metrics_size_2);
        // sample data size(3 + 4) = total source metrics metadata size
        assert!(
            (sample_3_total_size + sample_4_total_size) == total_source_metrics_metadata_size_2
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_source_metric_data_save_in_multi_thread() {
        let loop_cnt = 10;
        let mut hendle_vec = vec![];
        for idx in 0..loop_cnt {
            let hendle = tokio::spawn(async move {
                let json_value1 =
                    json!([{"name": format!("test_{}", idx), "value": 1.0}]).to_string();
                let mut source_metrics_data = SOURCE_METRICS_DATA.write().unwrap();
                let mut source_metrics_map = BTreeMap::new();
                source_metrics_map.insert(
                    Ulid::new().to_string(),
                    SourceMetrics {
                        json_value: json_value1,
                    },
                );
                source_metrics_data
                    .source_metrics
                    .insert(format!("metric_{}", idx), source_metrics_map);
                println!("idx : {}", idx);
            });
            hendle_vec.push(hendle);
        }
        for hendle in hendle_vec {
            if let Err(e) = hendle.await {
                println!("Error in handle: {:?}", e);
            }
        }
        let mut read_idx = 0;
        SOURCE_METRICS_DATA
            .read()
            .unwrap()
            .source_metrics
            .iter()
            .for_each(|(metric_id, map)| {
                println!(
                    "metric_id: {}, ulid: {}, name: {}",
                    metric_id,
                    map.first_key_value().unwrap().0,
                    map.first_key_value().unwrap().1.json_value
                );
                read_idx += 1;
            });
        assert_eq!(read_idx, loop_cnt);
    }
}
