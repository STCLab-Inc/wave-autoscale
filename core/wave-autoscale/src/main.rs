/**
* Wave Autoscale
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
use args::Args;
use clap::Parser;
use data_layer::{
    data_layer::{DataLayer, DataLayerNewParam},
    reader::{
        config_reader::read_config_file,
        yaml_reader::{read_yaml_file, ParserResult},
    },
};
use scaling_planner::ScalingPlanner;
use std::sync::Arc;
use tokio::task::JoinHandle;
mod args;
mod metric_adapter;
mod metric_store;
mod scaling_component;
mod scaling_planner;
mod util;

#[macro_use]
extern crate log;

const DEFAULT_PLAN_FILE: &str = "./plan.yaml";
const DEFAULT_CONFIG_FILE: &str = "./wave-config.yaml";
const DEFAULT_DB_URL: &str = "sqlite://wave.db";

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::init();

    // Initialize the array of handles to contain the handles of the spawned threads
    let mut async_handles: Vec<JoinHandle<()>> = vec![];

    // Parse command line arguments
    let args = Args::parse();

    // Read plans file that might not exist
    let plans_file: String;
    if args.plan.is_none() {
        info!(
            "No plans file specified, using default plans file: {}",
            DEFAULT_PLAN_FILE
        );
        plans_file = DEFAULT_PLAN_FILE.to_string();
    } else {
        plans_file = args.plan.unwrap();
        info!("Using plans file: {:?}", &plans_file);
    }

    let plans_file_result = read_yaml_file(plans_file);
    if plans_file_result.is_err() {
        let error = plans_file_result.as_ref().err().unwrap();
        error!("Error reading plans file: {}", error);
    } else {
        info!("Successfully read plans file");
    }
    let plans_file_result = match plans_file_result {
        Ok(plans_file_result) => plans_file_result,
        Err(_) => ParserResult {
            metric_definitions: vec![],
            scaling_component_definitions: vec![],
            scaling_plan_definitions: vec![],
            slo_definitions: vec![],
        },
    };

    // Create MetricStore(Arc<RwLock<HashMap<String, Value>>>)
    let shared_metric_store = metric_store::new_shared_metric_store();

    // Create MetricAdapterManager
    let mut metric_adapter_manager =
        metric_adapter::MetricAdapterManager::new(shared_metric_store.clone());

    // Add metric definitions to MetricAdapterManager
    let metric_definitions = plans_file_result.metric_definitions.clone();
    let metric_adapter_result = metric_adapter_manager.add_definitions(metric_definitions);
    if metric_adapter_result.is_err() {
        let error = metric_adapter_result.err().unwrap();
        error!("Error adding metric definitions: {}", error);
    } else {
        info!(
            "Successfully added metric definitions: {}",
            plans_file_result.metric_definitions.len()
        );
    }
    let metric_async_handles = metric_adapter_manager.run();
    async_handles.extend(metric_async_handles);
    info!("MetricAdapterManager started");

    // Create ScalingComponentManager
    let shared_scaling_component_manager =
        scaling_component::new_shared_scaling_component_manager();

    // If the writer of the scaling component manager is not released as soon as possible, the others will not be able to acquire the reader lock
    {
        // let cloned = shared_scaling_component_manager.clone();
        // let mut cloned_scaling_component_manager = cloned.write().await;
        let mut shared_scaling_component_manager_writer =
            shared_scaling_component_manager.write().await;
        let scaling_component_result = shared_scaling_component_manager_writer
            .add_definitions(plans_file_result.scaling_component_definitions.clone());
        if scaling_component_result.is_err() {
            let error = scaling_component_result.err().unwrap();
            error!("Error adding scaling component definitions: {}", error);
        } else {
            info!("Successfully added scaling component definitions");
        }
    }

    // Read config file
    let config_file: String;
    if args.config.is_none() {
        info!("No config file specified, using default config file: ./config.yaml");
        config_file = DEFAULT_CONFIG_FILE.to_string();
    } else {
        config_file = args.config.unwrap();
        info!("Using config file: {:?}", &config_file);
    }

    let config_file = read_config_file(config_file);

    let mut db_url: String = String::new();
    if config_file.is_err() {
        let error = config_file.err().unwrap();
        error!("Error reading config file: {}", error);
    } else {
        info!("Successfully read config file");
        let config_file = config_file.unwrap();
        let config_db_url = config_file.get("db_url");
        if config_db_url.is_none() {
            error!("No db_url specified in config file");
        } else {
            db_url = config_db_url.unwrap().as_str().unwrap().to_string();
            info!("Using db_url: {}", &db_url);
        }
    }

    // If db_url is empty, use default db_url
    if db_url.is_empty() {
        db_url = DEFAULT_DB_URL.to_string();
        info!("Using default db_url: {}", &db_url);
    }

    // Create DataLayer
    let data_layer = DataLayer::new(DataLayerNewParam {
        sql_url: db_url.clone(),
    })
    .await;
    let shared_data_layer = Arc::new(data_layer);

    // Create ScalingPlanner array
    let scaling_planners: Vec<ScalingPlanner> = plans_file_result
        .scaling_plan_definitions
        .iter()
        .map(|definition| {
            ScalingPlanner::new(
                definition.clone(),
                shared_metric_store.clone(),
                shared_scaling_component_manager.clone(),
                shared_data_layer.clone(),
            )
        })
        .collect();

    let number_of_scaling_planners = scaling_planners.len();
    // Run ScalingPlanners
    for scaling_planner in scaling_planners {
        let handle = scaling_planner.run();
        async_handles.push(handle);
    }
    info!("ScalingPlanners started: {}", number_of_scaling_planners);

    let async_handles_length = async_handles.len();
    // Keep this main thread alive until the program is terminated
    for handle in async_handles {
        handle.await.expect("Failed to join metric adapter manager");
    }
    println!("async_handles: {}", async_handles_length);
    if async_handles_length == 0 {
        info!("There is no plan to run");
    }
}
