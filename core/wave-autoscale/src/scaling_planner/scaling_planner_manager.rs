use crate::{metric_store::SharedMetricStore, scaling_component::SharedScalingComponentManager};

use super::ScalingPlanner;
use anyhow::Result;
use data_layer::{data_layer::DataLayer, ScalingPlanDefinition};
use std::{collections::HashMap, sync::Arc};
use tokio::{sync::RwLock, task::JoinHandle};
//
// PlannerManager
//
pub type SharedScalingPlannerManager = Arc<RwLock<ScalingPlannerManager>>;

#[derive(Default)]
pub struct ScalingPlannerManager {
    scaling_planners: HashMap<String, ScalingPlanner>,
}

impl ScalingPlannerManager {
    pub fn new() -> Self {
        ScalingPlannerManager {
            scaling_planners: HashMap::new(),
        }
    }
    pub fn new_shared() -> SharedScalingPlannerManager {
        Arc::new(RwLock::new(ScalingPlannerManager::new()))
    }

    // Factory method to create a scaling component.
    fn create_scaling_planner(
        &self,
        definition: ScalingPlanDefinition,
        metric_store: SharedMetricStore,
        scaling_component_manager: SharedScalingComponentManager,
        data_layer: Arc<DataLayer>,
    ) -> Result<ScalingPlanner> {
        Ok(ScalingPlanner::new(
            definition,
            metric_store,
            scaling_component_manager,
            data_layer,
        ))
    }

    pub fn add_definitions(
        &mut self,
        scaling_plan_definitions: Vec<ScalingPlanDefinition>,
        metric_store: SharedMetricStore,
        scaling_component_manager: SharedScalingComponentManager,
        data_layer: Arc<DataLayer>,
    ) -> Result<()> {
        for scaling_plan_definition in scaling_plan_definitions {
            let scaling_component = self.create_scaling_planner(
                scaling_plan_definition,
                metric_store.clone(),
                scaling_component_manager.clone(),
                data_layer.clone(),
            )?;
            self.add_scaling_component(scaling_component);
        }
        Ok(())
    }

    pub fn add_scaling_component(&mut self, scaling_planner: ScalingPlanner) {
        self.scaling_planners
            .insert(scaling_planner.get_id().to_string(), scaling_planner);
    }

    pub fn get_scaling_planners(&self) -> &HashMap<String, ScalingPlanner> {
        &self.scaling_planners
    }

    pub fn remove_all(&mut self) {
        self.scaling_planners.clear();
    }

    pub fn run(&mut self) {
        for scaling_planner in self.scaling_planners.values_mut() {
            scaling_planner.run();
        }
    }
    pub fn stop(&mut self) {
        for scaling_planner in self.scaling_planners.values_mut() {
            scaling_planner.stop();
        }
    }
}
