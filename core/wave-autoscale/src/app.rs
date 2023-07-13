/**
* App
*
* This is the main entry point of the Wave Autoscale program.
* It is responsible for parsing command line arguments, reading configuration files, and
* starting the metric adapters, scaling components, and scaling planners.
*
* The config file is responsible for configuring the program.
* The plans file is responsible for defining the metric definitions, scaling component
*
* The SharedMetricStore is responsible for storing the metrics.(shared between metric adapters and scaling planners)
* The MetricAdapterManager is responsible for managing the metric adapters.
*
* The SharedScalingComponentManager is responsible for managing the scaling components.(shared in the scaling planners)
*
* The DataLayer is responsible for storing the scaling actions and the scaling events.

* The metric adapters are responsible for collecting metrics from the metric sources.
* The scaling components are responsible for scaling the target resources.
* The scaling planners are responsible for planning the scaling actions.


* The metric sources are responsible for providing the metrics.
* The metric definitions are responsible for defining the metrics.
* The scaling component definitions are responsible for defining the scaling components.
* The scaling plan definitions are responsible for defining the scaling plans.
* The slo definitions are responsible for defining the service level objectives.
definitions, scaling plan definitions, and slo definitions.
* The command line arguments are responsible for configuring the program.
* The logger is responsible for logging.
* The util is responsible for providing utility functions.

* The scaling component manager is responsible for managing the scaling components.
*/
use crate::{
    args,
    metric_updater::{MetricUpdater, SharedMetricUpdater},
    scaling_component::{ScalingComponentManager, SharedScalingComponentManager},
    scaling_planner::scaling_planner_manager::{
        ScalingPlannerManager, SharedScalingPlannerManager,
    },
};
use args::Args;
use data_layer::data_layer::DataLayer;
use log::{debug, error};
use std::sync::Arc;
use tokio::time::sleep;
use utils::wave_config::WaveConfig;

pub struct App {
    args: Args,
    shared_data_layer: Arc<DataLayer>,
    shared_metric_updater: SharedMetricUpdater,
    shared_scaling_component_manager: SharedScalingComponentManager,
    shared_scaling_planner_manager: SharedScalingPlannerManager,
    autoscaling_history_remover_handle: Option<tokio::task::JoinHandle<()>>,
}

impl App {
    pub async fn new(args: Args) -> Self {
        // Read arguments
        let definition = args.definition.clone().unwrap_or_default();
        let config = args.config.clone().unwrap_or_default();

        // Read config file
        let config_result = WaveConfig::new(config.as_str());
        let db_url = config_result.common.db_url.clone();

        // Create DataLayer
        let data_layer = DataLayer::new(db_url.as_str(), definition.as_str()).await;
        // Do not need RwLock or Mutex because the DataLayer is read-only.
        let shared_data_layer = Arc::new(data_layer);

        // Create MetricUpdater
        let shared_metric_updater = MetricUpdater::new_shared(shared_data_layer.clone(), 1000);

        // Create MetricAdapterManager
        // let shared_metric_adapter_manager =
        //     metric_ad

        // Create ScalingComponentManager
        let shared_scaling_component_manager = ScalingComponentManager::new_shared();

        // Create ScalingPlanManager
        let shared_scaling_planner_manager = ScalingPlannerManager::new_shared(
            shared_data_layer.clone(),
            shared_metric_updater.clone(),
            shared_scaling_component_manager.clone(),
        );

        // Create App
        App {
            args,
            shared_data_layer,
            shared_metric_updater,
            shared_scaling_component_manager,
            shared_scaling_planner_manager,
            autoscaling_history_remover_handle: None,
        }
    }

    pub async fn run(&mut self) {
        // Scaling Component Manager
        // Scope for shared_scaling_component_manager_writer(RwLock)
        {
            // Reload scaling component definitions from DataLayer
            let scaling_component_definitions =
                self.shared_data_layer.get_all_scaling_components().await;

            // If there is an error, log it and return
            if scaling_component_definitions.is_err() {
                let error = scaling_component_definitions.err().unwrap();
                error!("Error getting scaling component definitions: {}", error);
                return;
            }

            let scaling_component_definitions = scaling_component_definitions.unwrap();
            let mut manager_writer = self.shared_scaling_component_manager.write().await;
            // TODO: Compare the number of scaling components before and after reloading.
            manager_writer.remove_all();
            let scaling_component_result =
                manager_writer.add_definitions(scaling_component_definitions);

            if scaling_component_result.is_err() {
                let error = scaling_component_result.err().unwrap();
                error!("Error adding scaling component definitions: {}", error);
                return;
            }
            debug!("Successfully added scaling component definitions");
        }

        // Metric Updater
        // Scope for shared_metric_updater_writer(RwLock)
        {
            let mut updater_writer = self.shared_metric_updater.write().await;
            updater_writer.run().await;
        }

        // Scaling Planner(managed by Vec)
        // Reload scaling plan definitions from DataLayer
        let plan_definitions = self.shared_data_layer.get_all_plans().await;
        if plan_definitions.is_err() {
            let error = plan_definitions.err().unwrap();
            error!("Error getting scaling plan definitions: {}", error);
            return;
        }
        let plan_definitions = plan_definitions.unwrap();
        let number_of_plans = plan_definitions.len();
        // Scope for shared_scaling_plan_manager_writer(RwLock)
        {
            let mut manager_writer = self.shared_scaling_planner_manager.write().await;
            manager_writer.remove_all();
            let scaling_plan_result = manager_writer.add_definitions(plan_definitions);
            if scaling_plan_result.is_err() {
                let error = scaling_plan_result.err().unwrap();
                error!("Error adding scaling plan definitions: {}", error);
                return;
            }
            debug!("Successfully added scaling plan definitions");
            manager_writer.run();
        }
        debug!("ScalingPlanners started: {}", number_of_plans);
    }

    pub async fn run_with_watching(&mut self) {
        debug!("run_with_watching");
        // Start the cron job to remove the old Autoscaling History
        let remove_autoscaling_history_duration = self.args.autoscaling_history_retention.clone();

        debug!(
            "remove_autoscaling_history_duration: {:?}",
            remove_autoscaling_history_duration
        );
        match remove_autoscaling_history_duration {
            Some(duration_string) if !duration_string.is_empty() => {
                self.run_autoscaling_history_cron_job(duration_string);
            }
            _ => {}
        }

        let watch_duration = self.args.watch_duration;
        let mut watch_receiver = self.shared_data_layer.watch(watch_duration);
        // Run this loop at once and then wait for changes
        let mut once = false;
        while !once || watch_receiver.changed().await.is_ok() {
            if once {
                let change = watch_receiver.borrow();
                debug!("DataLayer changed: {:?}", change);
            } else {
                once = true;
            }
            self.run().await;
        }
    }

    pub fn get_data_layer(&self) -> Arc<DataLayer> {
        self.shared_data_layer.clone()
    }

    pub fn get_scaling_component_manager(&self) -> SharedScalingComponentManager {
        self.shared_scaling_component_manager.clone()
    }

    pub fn get_scaling_planner_manager(&self) -> SharedScalingPlannerManager {
        self.shared_scaling_planner_manager.clone()
    }

    // Run the cron job to remove the old Autoscaling History
    pub fn run_autoscaling_history_cron_job(&mut self, duration_string: String) {
        self.stop_autoscaling_history_cron_job();
        debug!("Starting autoscaling history cron job: {}", duration_string);
        let duration = duration_str::parse(&duration_string);
        if duration.is_err() {
            error!("Error parsing duration string: {}", duration_string);
            return;
        }
        let duration = duration.unwrap();
        let duration = chrono::Duration::from_std(duration);
        if duration.is_err() {
            error!("Error converting duration: {}", duration_string);
            return;
        }
        let duration = duration.unwrap();
        let to_date = chrono::Utc::now() - duration;
        let data_layer = self.shared_data_layer.clone();
        let handle = tokio::spawn(async move {
            let result = data_layer.remove_old_autoscaling_history(to_date).await;
            if result.is_err() {
                let error = result.err().unwrap();
                error!("Error removing old autoscaling history: {}", error);
            }

            sleep(std::time::Duration::from_secs(60)).await;
        });
        self.autoscaling_history_remover_handle = Some(handle);
    }

    // Stop the cron job to remove the old Autoscaling History
    pub fn stop_autoscaling_history_cron_job(&mut self) {
        if self.autoscaling_history_remover_handle.is_some() {
            if let Some(handle) = self.autoscaling_history_remover_handle.take() {
                handle.abort();
                self.autoscaling_history_remover_handle = None;
            }
        }
    }
}
