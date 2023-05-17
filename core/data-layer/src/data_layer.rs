use anyhow::{anyhow, Result};
use sqlx::{
    any::{AnyKind, AnyPoolOptions, AnyQueryResult},
    AnyPool, Row,
};
use std::path::Path;
use uuid::Uuid;

use crate::{
    types::object_kind::ObjectKind, MetricDefinition, ScalingComponentDefinition,
    ScalingPlanDefinition,
};

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
            let db_id = Uuid::new_v4().to_string();

            let result = sqlx::query(query_string)
                .bind(db_id)
                .bind(metric.id.to_lowercase())
                .bind(metric.metric_kind.to_lowercase())
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
    // Get a metric from the database
    pub async fn get_metric_by_id(&self, db_id: String) -> Result<MetricDefinition> {
        let query_string = "SELECT db_id, id, metric_kind, metadata FROM metric WHERE db_id=?";
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
    // Add multiple scaling components to the database
    pub async fn add_scaling_components(
        &self,
        scaling_components: Vec<ScalingComponentDefinition>,
    ) -> Result<()> {
        // Define a pool variable that is a trait to pass to the execute function
        for scaling_component in scaling_components {
            let metadata_string = serde_json::to_string(&scaling_component.metadata).unwrap();
            let query_string =
                "INSERT INTO scaling_component (db_id, id, component_kind, metadata) VALUES (?,?,?,?)";
            let id = Uuid::new_v4().to_string();

            let result = sqlx::query(query_string)
                .bind(id)
                .bind(scaling_component.id)
                .bind(scaling_component.component_kind)
                .bind(metadata_string)
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
            "UPDATE scaling_component SET id=?, component_kind=?, metadata=? WHERE db_id=?";
        let result = sqlx::query(query_string)
            .bind(scaling_component.id)
            .bind(scaling_component.component_kind)
            .bind(metadata_string)
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
            let query_string = "INSERT INTO plan (db_id, id, title, plans) VALUES (?,?,?,?)";
            let id = Uuid::new_v4().to_string();

            let result = sqlx::query(query_string)
                .bind(id)
                .bind(plan.id)
                .bind(plan.title)
                .bind(plans_string)
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
        let query_string = "UPDATE plan SET id=?, title=?, plans=? WHERE db_id=?";
        let result = sqlx::query(query_string)
            .bind(plan.id)
            .bind(plan.title)
            .bind(plans_string)
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
}
