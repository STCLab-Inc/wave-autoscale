use std::sync::Arc;

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
use tokio::{task::JoinHandle};
mod args;
mod metric_adapter;
mod metric_store;
mod scaling_component;
mod scaling_planner;
mod util;

#[macro_use]
extern crate log;

const DEFAULT_PLAN_FILE: &str = "./plans.yaml";
const DEFAULT_CONFIG_FILE: &str = "./config.yaml";
const DEFAULT_DB_URL: &str = "sqlite://wave.db";

struct Config {
    db_url: String,
}

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::init();

    // Initialize async handles
    let mut handles: Vec<JoinHandle<()>> = vec![];

    // Parse command line arguments
    let args = Args::parse();

    let plans_file: String;
    if args.plans.is_none() {
        info!(
            "No plans file specified, using default plans file: {}",
            DEFAULT_PLAN_FILE
        );
        plans_file = DEFAULT_PLAN_FILE.to_string();
    } else {
        plans_file = args.plans.unwrap();
        info!("Using plans file: {:?}", &plans_file);
    }
    // read yaml file
    let plans_file_result = read_yaml_file(plans_file);
    if plans_file_result.is_err() {
        let error = plans_file_result.as_ref().err().unwrap();
        error!("Error reading plans file: {}", error);
    } else {
        info!("Successfully read config file");
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

    // create metric adapter manager
    let metric_store = metric_store::new_metric_store();
    let mut metric_adapter_manager =
        metric_adapter::MetricAdapterManager::new(metric_store.clone());
    let metric_definitions = plans_file_result.metric_definitions;
    let metric_adapter_result = metric_adapter_manager.add_definitions(metric_definitions);
    if metric_adapter_result.is_err() {
        let error = metric_adapter_result.as_ref().err().unwrap();
        error!("Error adding metric definitions: {}", error);
    } else {
        info!("Successfully added metric definitions");
    }
    let metric_handles = metric_adapter_manager.run();
    handles.extend(metric_handles);
    info!("Metric adapter manager started");

    // create scaling component manager
    let scaling_component_manager = scaling_component::new_scaling_component_manager();
    // If the writer of the scaling component manager is not released as soon as possible, the others will not be able to acquire the reader lock
    {
        let cloned = scaling_component_manager.clone();
        let mut cloned_scaling_component_manager = cloned.write().await;
        let scaling_component_result = cloned_scaling_component_manager
            .add_definitions(plans_file_result.scaling_component_definitions);
        if scaling_component_result.is_err() {
            let error = scaling_component_result.as_ref().err().unwrap();
            error!("Error adding scaling component definitions: {}", error);
        } else {
            info!("Successfully added scaling component definitions");
        }
    }

    let config_file: String;
    if args.config.is_none() {
        info!("No config file specified, using default config file: ./config.yaml");
        config_file = DEFAULT_CONFIG_FILE.to_string();
    } else {
        config_file = args.config.unwrap();
        info!("Using config file: {:?}", &config_file);
    }
    // read config file
    let config_file = read_config_file(config_file);
    let mut db_url: String = String::new();
    if config_file.is_err() {
        let error = config_file.as_ref().err().unwrap();
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

    // db_url in config
    if db_url.is_empty() {
        db_url = DEFAULT_DB_URL.to_string();
        info!("Using default db_url: {}", &db_url);
    }

    // create data layer
    let data_layer = DataLayer::new(DataLayerNewParam {
        sql_url: db_url.clone(),
    })
    .await;
    let data_layer = Arc::new(data_layer);

    // create scaling planner
    let scaling_planners: Vec<ScalingPlanner> = plans_file_result
        .scaling_plan_definitions
        .iter()
        .map(|definition| {
            ScalingPlanner::new(
                definition.clone(),
                metric_store.clone(),
                scaling_component_manager.clone(),
                Arc::clone(&data_layer),
            )
        })
        .collect();
    // run scaling planners
    for scaling_planner in scaling_planners {
        let handle = scaling_planner.run();
        handles.push(handle);
    }
    info!("Scaling planners started");
    // Keep this main thread alive until the program is terminated
    for handle in handles {
        handle.await.expect("Failed to join metric adapter manager");
    }
}
