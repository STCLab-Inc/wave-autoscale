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
use crate::{args, metric_adapter, metric_store, scaling_component, scaling_planner};
use args::Args;
use data_layer::{
    data_layer::{DataLayer, DataLayerNewParam},
    reader::{
        config_reader::read_config_file,
        yaml_reader::{read_yaml_file, ParserResult},
    },
};
use log::{error, info};
use scaling_planner::ScalingPlanner;
use std::sync::Arc;

const DEFAULT_PLAN_FILE: &str = "./plan.yaml";
const DEFAULT_CONFIG_FILE: &str = "./wave-config.yaml";
const DEFAULT_DB_URL: &str = "sqlite://wave.db";

async fn save_parser_result_into_data_layer(parser_result: &ParserResult, data_layer: &DataLayer) {
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

pub async fn run(args: &Args) {
    // Initialize the array of handles to contain the handles of the spawned threads
    // let mut async_handles: Vec<JoinHandle<()>> = vec![];

    let plan = args.plan.clone();
    let config = args.config.clone();
    // Read plans file that might not exist
    let plans_file: String;
    if plan.is_none() {
        info!(
            "No plans file specified, using default plans file: {}",
            DEFAULT_PLAN_FILE
        );
        plans_file = DEFAULT_PLAN_FILE.to_string();
    } else {
        plans_file = plan.unwrap();
        info!("Using plans file: {:?}", &plans_file);
    }

    let plan_file_result = read_yaml_file(plans_file);
    if plan_file_result.is_err() {
        let error = plan_file_result.as_ref().err().unwrap();
        error!("Error reading plans file: {}", error);
    } else {
        info!("Successfully read plans file");
    }
    let plan_file_result = match plan_file_result {
        Ok(plans_file_result) => plans_file_result,
        Err(_) => ParserResult {
            metric_definitions: vec![],
            scaling_component_definitions: vec![],
            scaling_plan_definitions: vec![],
            slo_definitions: vec![],
        },
    };

    // Read config file
    let config_file: String;
    if config.is_none() {
        info!("No config file specified, using default config file: ./config.yaml");
        config_file = DEFAULT_CONFIG_FILE.to_string();
    } else {
        config_file = config.unwrap();
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

    // Save definitions into DataLayer
    {
        let shared_data_layer = shared_data_layer.clone();
        save_parser_result_into_data_layer(&plan_file_result, &shared_data_layer).await;
    }

    // Create MetricStore(Arc<RwLock<HashMap<String, Value>>>)
    let shared_metric_store: Arc<
        tokio::sync::RwLock<std::collections::HashMap<String, serde_json::Value>>,
    > = metric_store::new_shared_metric_store();

    // Create MetricAdapterManager
    let mut metric_adapter_manager =
        metric_adapter::MetricAdapterManager::new(shared_metric_store.clone());

    // Create ScalingComponentManager
    let shared_scaling_component_manager =
        scaling_component::new_shared_scaling_component_manager();

    // // Create ScalingPlanner array
    // let scaling_planners: Vec<ScalingPlanner> = plan_file_result
    //     .scaling_plan_definitions
    //     .iter()
    //     .map(|definition| {
    //         ScalingPlanner::new(
    //             definition.clone(),
    //             shared_metric_store.clone(),
    //             shared_scaling_component_manager.clone(),
    //             shared_data_layer.clone(),
    //         )
    //     })
    //     .collect();

    // let number_of_scaling_planners = scaling_planners.len();
    // // Run ScalingPlanners
    // for scaling_planner in scaling_planners {
    //     let handle = scaling_planner.run();
    //     async_handles.push(handle);
    // }
    // info!("ScalingPlanners started: {}", number_of_scaling_planners);

    // let async_handles_length = async_handles.len();
    // // Keep this main thread alive until the program is terminated
    // for handle in async_handles {
    //     handle.await.expect("Failed to join metric adapter manager");
    // }
    // println!("async_handles: {}", async_handles_length);
    // if async_handles_length == 0 {
    //     info!("There is no plan to run");
    // }

    let mut watch_receiver = shared_data_layer.watch();
    let mut once = false;
    while !once || watch_receiver.changed().await.is_ok() {
        // Only print after the first change
        if once {
            let change = watch_receiver.borrow();
            info!("DataLayer changed: {:?}", change);
        } else {
            once = true;
        }

        // Metric Adapter Manager
        let metric_definitions = shared_data_layer.get_all_metrics().await;
        if metric_definitions.is_ok() {
            let metric_definitions = metric_definitions.unwrap();
            info!("metric_definitions: {:?}", metric_definitions);

            metric_adapter_manager.stop();
            metric_adapter_manager.remove_all_definitions();
            let metric_adapter_result = metric_adapter_manager.add_definitions(metric_definitions);
            if metric_adapter_result.is_err() {
                let error = metric_adapter_result.err().unwrap();
                error!("Error adding metric definitions: {}", error);
            } else {
                info!(
                    "Successfully added metric definitions: {}",
                    plan_file_result.metric_definitions.len()
                );
                let metric_async_handles = metric_adapter_manager.run();
            }
        } else {
            let error = metric_definitions.err().unwrap();
            error!("Error getting metric definitions: {}", error);
        }

        // Scaling Component Manager
        // If the writer of the scaling component manager is not released as soon as possible, the others will not be able to acquire the reader lock
        {
            // let cloned = shared_scaling_component_manager.clone();
            // let mut cloned_scaling_component_manager = cloned.write().await;
            let mut shared_scaling_component_manager_writer =
                shared_scaling_component_manager.write().await;
            let scaling_component_definitions =
                shared_data_layer.get_all_scaling_components().await;
            if scaling_component_definitions.is_ok() {
                let scaling_component_definitions = scaling_component_definitions.unwrap();
                shared_scaling_component_manager_writer.remove_all();
                let scaling_component_result = shared_scaling_component_manager_writer
                    .add_definitions(scaling_component_definitions);
                if scaling_component_result.is_err() {
                    let error = scaling_component_result.err().unwrap();
                    error!("Error adding scaling component definitions: {}", error);
                } else {
                    info!("Successfully added scaling component definitions");
                }
            }
        }

        // Scaling Planner
        // Create ScalingPlanner array
        let plan_definitions = shared_data_layer.get_all_plans().await;
        if plan_definitions.is_ok() {
            let plan_definitions = plan_definitions.unwrap();
            let scaling_planners: Vec<ScalingPlanner> = plan_definitions
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
                // async_handles.push(handle);
            }
            info!("ScalingPlanners started: {}", number_of_scaling_planners);
        }
    }
}
