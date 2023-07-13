use crate::{
    metric_updater::SharedMetricUpdater, scaling_component::SharedScalingComponentManager,
};

use super::ScalingPlanner;
use anyhow::Result;
use data_layer::{data_layer::DataLayer, ScalingPlanDefinition};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
//
// PlannerManager
//
pub type SharedScalingPlannerManager = Arc<RwLock<ScalingPlannerManager>>;

pub struct ScalingPlannerManager {
    scaling_planners: HashMap<String, ScalingPlanner>,
    data_layer: Arc<DataLayer>,
    metric_updater: SharedMetricUpdater,
    scaling_component_manager: SharedScalingComponentManager,
}

impl ScalingPlannerManager {
    pub fn new(
        data_layer: Arc<DataLayer>,
        metric_updater: SharedMetricUpdater,
        scaling_component_manager: SharedScalingComponentManager,
    ) -> Self {
        ScalingPlannerManager {
            scaling_planners: HashMap::new(),
            data_layer,
            metric_updater,
            scaling_component_manager,
        }
    }
    pub fn new_shared(
        data_layer: Arc<DataLayer>,
        metric_updater: SharedMetricUpdater,
        scaling_component_manager: SharedScalingComponentManager,
    ) -> SharedScalingPlannerManager {
        Arc::new(RwLock::new(ScalingPlannerManager::new(
            data_layer,
            metric_updater,
            scaling_component_manager,
        )))
    }

    // Factory method to create a scaling component.
    fn create_scaling_planner(&self, definition: ScalingPlanDefinition) -> Result<ScalingPlanner> {
        Ok(ScalingPlanner::new(
            definition,
            self.metric_updater.clone(),
            self.scaling_component_manager.clone(),
            self.data_layer.clone(),
        ))
    }

    pub fn add_definitions(
        &mut self,
        scaling_plan_definitions: Vec<ScalingPlanDefinition>,
    ) -> Result<()> {
        for scaling_plan_definition in scaling_plan_definitions {
            let scaling_component = self.create_scaling_planner(scaling_plan_definition)?;
            self.add_scaling_component(scaling_component);
        }
        Ok(())
    }

    pub fn add_scaling_component(&mut self, scaling_planner: ScalingPlanner) {
        self.scaling_planners
            .insert(scaling_planner.get_id(), scaling_planner);
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
