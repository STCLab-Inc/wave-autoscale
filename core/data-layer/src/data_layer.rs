use crate::{
    reader::wave_definition_reader::{read_definition_yaml_file, ParserResult},
    types::{
        autoscaling_history_definition::AutoscalingHistoryDefinition, object_kind::ObjectKind,
    },
    MetricDefinition, ScalingComponentDefinition, ScalingPlanDefinition,
};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sqlx::{
    any::{AnyKind, AnyPoolOptions, AnyQueryResult},
    AnyPool, Row,
};
use std::path::Path;
use tokio::sync::watch;
use uuid::Uuid;

const DEFAULT_DEFINITION_PATH: &str = "./plan.yaml";
const DEFAULT_DB_URL: &str = "sqlite://wave.db";

#[derive(Debug)]
pub struct DataLayer {
    // Pool is a connection pool to the database. Postgres, Mysql, SQLite supported.
    pool: AnyPool,
}

impl DataLayer {
    pub async fn new(sql_url: &str, definition_path: &str) -> Self {
        let sql_url = if sql_url.is_empty() {
            DEFAULT_DB_URL
        } else {
            sql_url
        };

        let data_layer = DataLayer {
            pool: DataLayer::get_pool(sql_url).await,
        };
        data_layer.migrate().await;

        // TODO: Validate the definition file before loading it into the database
        if data_layer
            .load_definition_file_into_database(definition_path)
            .await
            .is_err()
        {
            // If DataLayer fails to load the definition file, it's not safe to continue. So we panic here
            panic!("Failed to load definition file into database");
        }
        data_layer
    }

    async fn load_definition_file_into_database(&self, definition_path: &str) -> Result<()> {
        // Get the definition path not
        let definition_path = if definition_path.is_empty() {
            DEFAULT_DEFINITION_PATH
        } else {
            definition_path
        };
        // Parse the plan_file
        let parser_result = read_definition_yaml_file(definition_path);
        if parser_result.is_err() {
            return Err(anyhow!(
                "Failed to parse the definition file: {:?}",
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

        AnyPoolOptions::new()
            .max_connections(5)
            .connect(sql_url)
            .await
            .unwrap()
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
    pub fn watch(&self, watch_duration: u64) -> watch::Receiver<String> {
        let (notify_sender, notify_receiver) = watch::channel(String::new());
        let pool = self.pool.clone();
        tokio::spawn(async move {
            let mut lastest_updated_at_hash: String = String::new();
            loop {
                println!("Watching...");
                let query_string =
                    "SELECT updated_at FROM metric ORDER BY updated_at DESC LIMIT 1; SELECT updated_at FROM scaling_component ORDER BY updated_at DESC LIMIT 1; SELECT updated_at FROM plan ORDER BY updated_at DESC LIMIT 1;";
                let result_string = sqlx::query(query_string).fetch_all(&pool).await.unwrap();
                let mut updated_at_hash_string: String = String::new();
                for row in &result_string {
                    let updated_at: String = row.get(0);
                    println!("{:?}", updated_at);
                    updated_at_hash_string.push_str(&updated_at);
                }
                if lastest_updated_at_hash != updated_at_hash_string {
                    // Send signals after the first time
                    if !lastest_updated_at_hash.is_empty() {
                        println!("is not empty");
                        let timestamp = Utc::now().to_rfc3339();
                        let result = notify_sender.send(timestamp);
                        if result.is_err() {
                            error!(
                                "Failed to send notify signal: {}",
                                result.err().unwrap().to_string()
                            );
                        }
                    }
                    lastest_updated_at_hash = updated_at_hash_string;
                    println!("Updated at hash changed");
                }
                // 1 second
                tokio::time::sleep(tokio::time::Duration::from_secs(watch_duration)).await;
            }
        });
        notify_receiver
    }

    // Add multiple metrics to the database
    pub async fn add_metrics(&self, metrics: Vec<MetricDefinition>) -> Result<()> {
        // Define a pool variable that is a trait to pass to the execute function
        for metric in metrics {
            let metadata_string = serde_json::to_string(&metric.metadata).unwrap();
            let query_string =
                "INSERT INTO metric (db_id, id, collector, metric_kind, metadata, created_at, updated_at) VALUES (?,?,?,?,?,?,?) ON CONFLICT (id) DO UPDATE SET (collector, metric_kind, metadata, updated_at) = (?,?,?,?)";
            let db_id = Uuid::new_v4().to_string();
            let updated_at = Utc::now();
            let result = sqlx::query(query_string)
                // Values for insert
                .bind(db_id)
                .bind(metric.id.to_lowercase())
                .bind(metric.collector.to_lowercase())
                .bind(metric.metric_kind.to_lowercase())
                .bind(metadata_string.clone())
                .bind(updated_at)
                .bind(updated_at)
                // Values for update
                .bind(metric.collector.to_lowercase())
                .bind(metric.metric_kind.to_lowercase())
                .bind(metadata_string.clone())
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
        let query_string = "SELECT db_id, id, collector, metric_kind, metadata FROM metric";
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
                collector: row.get("collector"),
                metric_kind: row.get("metric_kind"),
                metadata: serde_json::from_str(row.get("metadata")).unwrap(),
            });
        }
        Ok(metrics)
    }
    // Get a metric from the database
    pub async fn get_metric_by_id(&self, db_id: String) -> Result<MetricDefinition> {
        let query_string =
            "SELECT db_id, id, collector, metric_kind, metadata FROM metric WHERE db_id=?";
        let result = sqlx::query(query_string)
            .bind(db_id)
            .fetch_one(&self.pool)
            .await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();
        let metric = MetricDefinition {
            kind: ObjectKind::Metric,
            db_id: result.get("db_id"),
            id: result.get("id"),
            collector: result.get("collector"),
            metric_kind: result.get("metric_kind"),
            metadata: serde_json::from_str(result.get("metadata")).unwrap(),
        };
        Ok(metric)
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
        let query_string =
            "UPDATE metric SET id=?, collector=?, metric_kind=?, metadata=?, updated_at=? WHERE db_id=?";
        let updated_at = Utc::now();
        let result = sqlx::query(query_string)
            // SET
            .bind(metric.id)
            .bind(metric.collector)
            .bind(metric.metric_kind)
            .bind(metadata_string)
            .bind(updated_at)
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
                "INSERT INTO scaling_component (db_id, id, component_kind, metadata, created_at, updated_at) VALUES (?,?,?,?,?,?) ON CONFLICT (id) DO UPDATE SET (metadata, updated_at) = (?,?)";
            let id = Uuid::new_v4().to_string();
            let updated_at = Utc::now();
            let result = sqlx::query(query_string)
                // Values for insert
                .bind(id)
                .bind(scaling_component.id)
                .bind(scaling_component.component_kind)
                .bind(metadata_string.clone())
                .bind(updated_at)
                .bind(updated_at)
                // Values for update
                .bind(metadata_string.clone())
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
        let query_string = "SELECT db_id, id, component_kind, metadata FROM scaling_component";
        let result = sqlx::query(query_string).fetch_all(&self.pool).await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();
        for row in result {
            scaling_components.push(ScalingComponentDefinition {
                kind: ObjectKind::ScalingComponent,
                db_id: row.get("db_id"),
                id: row.get("id"),
                component_kind: row.get("component_kind"),
                metadata: serde_json::from_str(row.get("metadata")).unwrap(),
            });
        }
        Ok(scaling_components)
    }
    // Get a scaling component from the database
    pub async fn get_scaling_component_by_id(
        &self,
        db_id: String,
    ) -> Result<ScalingComponentDefinition> {
        let query_string =
            "SELECT db_id, id, component_kind, metadata FROM scaling_component WHERE db_id=?";
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
        let query_string = "DELETE FROM scaling_component WHERE db_id=?";
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
            "UPDATE scaling_component SET id=?, component_kind=?, metadata=?, updated_at=? WHERE db_id=?";
        let updated_at = Utc::now();
        let result = sqlx::query(query_string)
            // SET
            .bind(scaling_component.id)
            .bind(scaling_component.component_kind)
            .bind(metadata_string)
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
            let query_string = "INSERT INTO plan (db_id, id, title, plans, created_at, updated_at) VALUES (?,?,?,?,?,?) ON CONFLICT (id) DO UPDATE SET (title, plans, updated_at) = (?,?,?)";
            let id = Uuid::new_v4().to_string();
            let updated_at = Utc::now();
            let result = sqlx::query(query_string)
                // Values for insert
                .bind(id)
                .bind(plan.id)
                .bind(plan.title.clone())
                .bind(plans_string.clone())
                .bind(updated_at)
                .bind(updated_at)
                // Values for update
                .bind(plan.title.clone())
                .bind(plans_string.clone())
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
        let query_string = "SELECT db_id, id, title, plans, priority FROM plan";
        let result = sqlx::query(query_string).fetch_all(&self.pool).await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();
        for row in result {
            plans.push(ScalingPlanDefinition {
                kind: ObjectKind::ScalingPlan,
                db_id: row.get("db_id"),
                id: row.get("id"),
                title: row.get("title"),
                plans: serde_json::from_str(row.get("plans")).unwrap(),
            });
        }
        Ok(plans)
    }
    // Get a plan from the database
    pub async fn get_plan_by_id(&self, db_id: String) -> Result<ScalingPlanDefinition> {
        let query_string = "SELECT db_id, id, title, plans FROM plan WHERE db_id=?";
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
            title: result.get("title"),
            plans: serde_json::from_str(result.get("plans")).unwrap(),
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
        let query_string = "DELETE FROM plan WHERE db_id=?";
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
        let query_string = "UPDATE plan SET id=?, title=?, plans=?, updated_at=? WHERE db_id=?";
        let updated_at = Utc::now();
        let result = sqlx::query(query_string)
            // SET
            .bind(plan.id)
            .bind(plan.title)
            .bind(plans_string)
            .bind(updated_at)
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
        let query_string = "INSERT INTO autoscaling_history (id, plan_db_id, plan_id, plan_item_json, metric_values_json, metadata_values_json, fail_message, created_at) VALUES (?,?,?,?,?,?,?,?)";
        let id = Uuid::new_v4().to_string();
        let result = sqlx::query(query_string)
            // INTO
            .bind(id)
            .bind(autoscaling_history.plan_db_id)
            .bind(autoscaling_history.plan_id)
            .bind(autoscaling_history.plan_item_json)
            .bind(autoscaling_history.metric_values_json)
            .bind(autoscaling_history.metadata_values_json)
            .bind(autoscaling_history.fail_message)
            .bind(autoscaling_history.created_at)
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
        let query_string = "SELECT id, plan_db_id, plan_id, plan_item_json, metric_values_json, metadata_values_json, fail_message, created_at FROM autoscaling_history WHERE plan_id=?";
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
                created_at: row.get("created_at"),
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
        // TODO: DRY - query_string
        let query_string = "SELECT id, plan_db_id, plan_id, plan_item_json, metric_values_json, metadata_values_json, fail_message, created_at FROM autoscaling_history WHERE created_at BETWEEN ? AND ?";
        let result = sqlx::query(query_string)
            .bind(from_date)
            .bind(to_date)
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
                created_at: row.get("created_at"),
            });
        }
        Ok(autoscaling_history)
    }
    // Remove the old AutoscalingHistory from the database
    pub async fn remove_old_autoscaling_history(&self, to_date: DateTime<Utc>) -> Result<()> {
        let query_string = "DELETE FROM autoscaling_history WHERE created_at < ?";
        let result = sqlx::query(query_string)
            .bind(to_date)
            .execute(&self.pool)
            .await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        Ok(())
    }
}
