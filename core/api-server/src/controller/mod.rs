pub mod autoscaling_history_controller;
pub mod definition_controller;
pub mod inflow_controller;
pub mod metric_controller;
pub mod metrics_receiver_controller;
pub mod plan_controller;
pub mod scaling_component_controller;

pub use autoscaling_history_controller::init as init_autoscaling_history_controller;
pub use definition_controller::init as init_definition_controller;
pub use inflow_controller::init as init_inflow_controller;
pub use metric_controller::init as init_metric_controller;
pub use metrics_receiver_controller::init as init_metrics_receiver_controller;
pub use plan_controller::init as init_plan_controller;
pub use scaling_component_controller::init as init_scaling_component_controller;
