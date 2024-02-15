use super::DataLayer;
use crate::{
    types::object_kind::ObjectKind,
    values_map::{apply_values_map, get_values_map},
    ScalingPlanDefinition,
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde_json::json;
use sqlx::{any::AnyQueryResult, Row};
use std::collections::HashMap;
use uuid::Uuid;

impl DataLayer {
    // Add multiple plans to the database
    pub async fn add_plans(&self, plans: Vec<ScalingPlanDefinition>) -> Result<()> {
        // Define a pool variable that is a trait to pass to the execute function
        for plan in plans {
            let variables_string = serde_json::to_string(&plan.variables).unwrap();
            let plans_string = serde_json::to_string(&plan.plans).unwrap();
            let metatdata_string = serde_json::to_string(&plan.metadata).unwrap();
            let query_string = "INSERT INTO plan (db_id, id, metadata, variables, plans, enabled, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT (id) DO UPDATE SET (metadata, variables, plans, enabled, updated_at) = ($9, $10, $11, $12, $13)";
            let id = Uuid::new_v4().to_string();
            let updated_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            let result = sqlx::query(query_string)
                // Values for insert
                .bind(id)
                .bind(plan.id)
                .bind(metatdata_string.clone())
                .bind(variables_string.clone())
                .bind(plans_string.clone())
                .bind(plan.enabled)
                .bind(updated_at.clone())
                .bind(updated_at.clone())
                // Values for update
                .bind(metatdata_string.clone())
                .bind(variables_string.clone())
                .bind(plans_string.clone())
                .bind(plan.enabled)
                .bind(updated_at.clone())
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
        let query_string =
            "SELECT db_id, id, variables, plans, priority, metadata, enabled FROM plan";
        let result = sqlx::query(query_string).fetch_all(&self.pool).await;
        if result.is_err() {
            return Err(anyhow!(result.err().unwrap().to_string()));
        }
        let result = result.unwrap();

        let variable_mapper_data = get_values_map();

        for row in result {
            let mut metadata = HashMap::new();
            let metadata_row = row.try_get::<&str, _>("metadata");
            if metadata_row.is_ok() {
                let metadata_str = metadata_row.unwrap();
                let metadata_json =
                    apply_values_map(metadata_str.to_string(), &variable_mapper_data);
                if metadata_json.is_ok() {
                    let json = serde_json::from_str(metadata_json.unwrap().as_str());
                    if json.is_ok() {
                        metadata = json.unwrap();
                    }
                }
            }

            let mut variables = HashMap::new();
            let variables_row = row.try_get::<&str, _>("variables");
            if variables_row.is_ok() {
                let json = serde_json::from_str(variables_row.unwrap());
                if json.is_ok() {
                    variables = json.unwrap();
                }
            }

            let mut plan_items = Vec::new();
            let plans_row = row.try_get::<&str, _>("plans");
            if plans_row.is_ok() {
                let json = serde_json::from_str(plans_row.unwrap());
                if json.is_ok() {
                    plan_items = json.unwrap();
                }
            }

            plans.push(ScalingPlanDefinition {
                kind: ObjectKind::ScalingPlan,
                db_id: row.try_get::<String, _>("db_id")?,
                id: row.try_get::<String, _>("id")?,
                metadata,
                variables,
                plans: plan_items,
                enabled: row.try_get::<bool, _>("enabled").unwrap_or(false),
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
            "SELECT db_id, id, variables, plans, priority, metadata, enabled, created_at, updated_at FROM plan";
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
                "variables": serde_json::from_str::<serde_json::Value>(row.try_get::<String, _>("variables")?.as_str())?,
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
        let query_string =
            "SELECT db_id, id, metadata, variables, plans, enabled FROM plan WHERE db_id=$1";
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
            variables: serde_json::from_str(result.get("variables")).unwrap(),
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
        let updated_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
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
    async fn test_get_all_plans_json() {
        let data_layer = get_data_layer_with_sqlite().await;
        test_get_all_plans_json_with_data_layer(data_layer).await;

        let data_layer = get_data_layer_with_postgres().await;
        test_get_all_plans_json_with_data_layer(data_layer).await;
    }
    async fn test_get_all_plans_json_with_data_layer(data_layer: DataLayer) {
        let scaling_plan_definition = ScalingPlanDefinition {
            kind: ObjectKind::ScalingPlan,
            db_id: Ulid::new().to_string(),
            id: "scaling_plan_test_id".to_string(),
            metadata: HashMap::new(),
            variables: HashMap::new(),
            plans: vec![],
            enabled: true,
        };
        // add plans
        let add_plans_result: std::prelude::v1::Result<(), anyhow::Error> = data_layer
            .add_plans(vec![scaling_plan_definition.clone()])
            .await;
        assert!(add_plans_result.is_ok());

        // read plans
        let scaling_plans = data_layer.get_all_plans_json().await;
        scaling_plans.unwrap().iter().for_each(|scaling_plan| {
            if scaling_plan.get("id").unwrap() == "scaling_plan_test_id" {
                assert_eq!(scaling_plan.get("enabled").unwrap(), &true);
            }
        });
    }
}
