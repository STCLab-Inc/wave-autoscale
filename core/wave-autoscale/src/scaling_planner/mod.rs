use data_layer::ScalingPlanDefinition;

pub struct ScalingPlanner {
    scaling_plan: ScalingPlanDefinition,
}

impl ScalingPlanner {
    pub fn new(scaling_plan: ScalingPlanDefinition) -> Self {
        ScalingPlanner { scaling_plan }
    }
}
