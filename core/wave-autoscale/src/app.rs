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
    metric_adapter::{self, MetricAdapterManager},
    metric_store::{self, SharedMetricStore},
    scaling_component::{ScalingComponentManager, SharedScalingComponentManager},
    scaling_planner::scaling_planner_manager::{
        ScalingPlannerManager, SharedScalingPlannerManager,
    },
};
use args::Args;
use data_layer::{
    data_layer::{DataLayer, DataLayerNewParam},
    reader::{
        config_reader::read_config_file,
        yaml_reader::{read_yaml_file, ParserResult},
    },
};
use log::{debug, error};
use std::sync::Arc;

const DEFAULT_PLAN_FILE: &str = "./plan.yaml";
const DEFAULT_DB_URL: &str = "sqlite://wave.db";

pub struct App {
    args: Args,
    shared_data_layer: Arc<DataLayer>,
    shared_metric_store: SharedMetricStore,
    metric_adapter_manager: MetricAdapterManager,
    shared_scaling_component_manager: SharedScalingComponentManager,
    shared_scaling_planner_manager: SharedScalingPlannerManager,
}

impl App {
    pub async fn new(args: Args) -> Self {
        // Read command line arguments
        let plan = args.plan.clone();
        let config = args.config.clone();
        let watch_duration = args.watch_duration;

        // Read plans file that might not exist
        let plan_file = App::get_plan_file_path(plan);
        let parse_result = App::parse_plan_file(&plan_file);

        // Read config file
        let config_result = read_config_file(config);

        // DB_URL from config file
        let mut db_url: String = String::new();

        let config_db_url = config_result.get("DB_URL");
        if config_db_url.is_none() {
            debug!("No db_url specified in config file");
        } else {
            db_url = config_db_url.unwrap().as_str().unwrap().to_string();
            debug!("Using db_url: {}", &db_url);
        }

        // If db_url is empty, use default db_url
        if db_url.is_empty() {
            db_url = DEFAULT_DB_URL.to_string();
            debug!("Using default db_url: {}", &db_url);
        }

        // Create DataLayer
        let data_layer = DataLayer::new(DataLayerNewParam {
            sql_url: db_url.clone(),
            watch_duration,
        })
        .await;
        let shared_data_layer = Arc::new(data_layer);

        // Save definitions into DataLayer
        {
            let shared_data_layer = shared_data_layer.clone();
            App::save_parser_result_into_data_layer(&parse_result, &shared_data_layer).await;
        }

        // Create MetricStore(Arc<RwLock<HashMap<String, Value>>>)
        let shared_metric_store: Arc<
            tokio::sync::RwLock<std::collections::HashMap<String, serde_json::Value>>,
        > = metric_store::new_shared();

        // Create MetricAdapterManager
        let metric_adapter_manager =
            metric_adapter::MetricAdapterManager::new(shared_metric_store.clone());

        // Create ScalingComponentManager
        let shared_scaling_component_manager = ScalingComponentManager::new_shared();

        // Create ScalingPlanManager
        let shared_scaling_planner_manager = ScalingPlannerManager::new_shared();

        // Create App
        App {
            args,
            shared_data_layer,
            shared_metric_store,
            metric_adapter_manager,
            shared_scaling_component_manager,
            shared_scaling_planner_manager,
        }
    }
    async fn save_parser_result_into_data_layer(
        parser_result: &ParserResult,
        data_layer: &DataLayer,
    ) {
        // Save definitions into DataLayer
        let metric_definitions = parser_result.metric_definitions.clone();
        let metric_definitions_result = data_layer.add_metrics(metric_definitions).await;
        if metric_definitions_result.is_err() {
            error!("Failed to save metric definitions into DataLayer");
        }

        // Save definitions into DataLayer
        let scaling_component_definitions = parser_result.scaling_component_definitions.clone();
        let scaling_component_definitions_result = data_layer
            .add_scaling_components(scaling_component_definitions)
            .await;
        if scaling_component_definitions_result.is_err() {
            error!("Failed to save scaling component definitions into DataLayer");
        }

        // Save definitions into DataLayer
        let scaling_plan_definitions = parser_result.scaling_plan_definitions.clone();
        let scaling_plan_definitions_result = data_layer.add_plans(scaling_plan_definitions).await;
        if scaling_plan_definitions_result.is_err() {
            error!("Failed to save scaling plan definitions into DataLayer");
        }
    }

    fn get_plan_file_path(plan: Option<String>) -> String {
        let plan_file: String;
        if plan.is_none() {
            debug!(
                "No plans file specified, using default plans file: {}",
                DEFAULT_PLAN_FILE
            );
            plan_file = DEFAULT_PLAN_FILE.to_string();
        } else {
            plan_file = plan.unwrap();
            debug!("Using plans file: {:?}", &plan_file);
        }
        plan_file
    }

    fn parse_plan_file(plan_file: &String) -> ParserResult {
        // Parse the plan_file
        let parse_result = read_yaml_file(plan_file);
        if parse_result.is_err() {
            let error = parse_result.as_ref().err().unwrap();
            debug!("Error reading plans file: {}", error);
        } else {
            debug!("Successfully read plans file");
        }

        match parse_result {
            Ok(plans_file_result) => plans_file_result,
            Err(_) => ParserResult {
                metric_definitions: vec![],
                scaling_component_definitions: vec![],
                scaling_plan_definitions: vec![],
                slo_definitions: vec![],
            },
        }
    }

    pub async fn run(&mut self) {
        // Reload metric definitions from DataLayer
        let metric_definitions = self.shared_data_layer.get_all_metrics().await;
        if metric_definitions.is_ok() {
            let metric_definitions = metric_definitions.unwrap();
            let number_of_metric_definitions = metric_definitions.len();
            debug!("metric_definitions: {:?}", metric_definitions);
            self.metric_adapter_manager.stop();
            self.metric_adapter_manager.remove_all_definitions();

            let metric_adapter_result = self
                .metric_adapter_manager
                .add_definitions(metric_definitions);
            if metric_adapter_result.is_err() {
                let error = metric_adapter_result.err().unwrap();
                error!("Error adding metric definitions: {}", error);
            } else {
                debug!(
                    "Successfully added metric definitions: {}",
                    number_of_metric_definitions
                );
                self.metric_adapter_manager.run();
            }
        } else {
            let error = metric_definitions.err().unwrap();
            error!("Error getting metric definitions: {}", error);
        }

        // Scaling Component Manager
        // Scope for shared_scaling_component_manager_writer(RwLock)
        {
            // Reload scaling component definitions from DataLayer
            let scaling_component_definitions =
                self.shared_data_layer.get_all_scaling_components().await;
            if scaling_component_definitions.is_ok() {
                let scaling_component_definitions = scaling_component_definitions.unwrap();

                let mut shared_scaling_component_manager_writer =
                    self.shared_scaling_component_manager.write().await;
                shared_scaling_component_manager_writer.remove_all();
                let scaling_component_result = shared_scaling_component_manager_writer
                    .add_definitions(scaling_component_definitions);

                if scaling_component_result.is_err() {
                    let error = scaling_component_result.err().unwrap();
                    error!("Error adding scaling component definitions: {}", error);
                } else {
                    debug!("Successfully added scaling component definitions");
                }
            } else {
                let error = scaling_component_definitions.err().unwrap();
                error!("Error getting scaling component definitions: {}", error);
            }
        }

        // Scaling Planner(managed by Vec)
        // Reload scaling plan definitions from DataLayer
        let plan_definitions = self.shared_data_layer.get_all_plans().await;
        if plan_definitions.is_ok() {
            let plan_definitions = plan_definitions.unwrap();
            let number_of_plans = plan_definitions.len();
            // Scope for shared_scaling_plan_manager_writer(RwLock)
            {
                let mut shared_scaling_plan_manager_writer =
                    self.shared_scaling_planner_manager.write().await;
                shared_scaling_plan_manager_writer.remove_all();
                let scaling_plan_result = shared_scaling_plan_manager_writer.add_definitions(
                    plan_definitions,
                    self.shared_metric_store.clone(),
                    self.shared_scaling_component_manager.clone(),
                    self.shared_data_layer.clone(),
                );
                if scaling_plan_result.is_err() {
                    let error = scaling_plan_result.err().unwrap();
                    error!("Error adding scaling plan definitions: {}", error);
                } else {
                    debug!("Successfully added scaling plan definitions");
                    shared_scaling_plan_manager_writer.run();
                }
            }
            debug!("ScalingPlanners started: {}", number_of_plans);
        } else {
            let error = plan_definitions.err().unwrap();
            error!("Error getting scaling plan definitions: {}", error);
        }
    }

    pub async fn run_with_watching(&mut self) {
        let mut watch_receiver = self.shared_data_layer.watch();
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

    pub fn get_metric_adapter_manager(&self) -> &MetricAdapterManager {
        &self.metric_adapter_manager
    }

    pub fn get_scaling_component_manager(&self) -> SharedScalingComponentManager {
        self.shared_scaling_component_manager.clone()
    }

    pub fn get_scaling_planner_manager(&self) -> SharedScalingPlannerManager {
        self.shared_scaling_planner_manager.clone()
    }
}
