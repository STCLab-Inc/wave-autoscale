/**
 * Wave Autoscale
 */
mod app;
mod args;
mod metric_collector_manager;
mod metric_updater;
mod scaling_component;
mod scaling_planner;
mod util;

use api_server::app::run_api_server;
use args::Args;
use clap::Parser;
use data_layer::data_layer::DataLayer;
use log::error;
use metric_collector_manager::MetricCollectorManager;
use std::sync::Arc;
use tokio::sync::watch;
use utils::wave_config::WaveConfig;

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::init();

    // Parse command line arguments
    // Separate function to allow for unit testing
    let args = Args::parse();

    // Configuration
    let wave_config = WaveConfig::new(args.config.as_str());

    // DataLayer
    let db_url = wave_config.db_url.clone();
    let data_layer = DataLayer::new(db_url.as_str()).await;
    // Do not need RwLock or Mutex because the DataLayer is read-only.
    let shared_data_layer = Arc::new(data_layer);
    shared_data_layer.sync(args.definition.as_str()).await;

    // MetricCollectorManager
    let output_url = format!(
        "http://{}:{}/api/metrics-receiver",
        wave_config.host, wave_config.port
    );
    let metric_collector_manager =
        MetricCollectorManager::new(wave_config.collectors_info.as_str(), &output_url);

    // Run API Server
    let shared_data_layer_for_api_server = shared_data_layer.clone();
    let wave_config_for_api_server = wave_config.clone();
    let _api_server_handle = tokio::spawn(async move {
        let _ = run_api_server(
            wave_config_for_api_server,
            shared_data_layer_for_api_server.clone(),
        );
    });

    // Run the main application(controller)
    let mut app = app::App::new(wave_config.clone(), shared_data_layer.clone()).await;

    // Remove autoscaling history
    if !wave_config.autoscaling_history_retention.is_empty() {
        app.run_autoscaling_history_cron_job(wave_config.autoscaling_history_retention);
    }

    // Watch the definition file
    let watch_duration = wave_config.watch_definition_duration;
    let mut watch_receiver: Option<watch::Receiver<String>> = if watch_duration != 0 {
        Some(shared_data_layer.watch_definitions(watch_duration))
    } else {
        None
    };

    // Run this loop at once and then wait for changes
    let mut once = false;
    while !once
        || (watch_receiver.is_some() && watch_receiver.as_mut().unwrap().changed().await.is_ok())
    {
        if once {
            // let change = watch_receiver.as_mut().unwrap().borrow();
        } else {
            once = true;
        }

        // Update metric collectors
        let shared_data_layer = shared_data_layer.clone();
        let metric_definitions = shared_data_layer.get_all_metrics().await;
        if metric_definitions.is_err() {
            error!("Failed to get metric definitions");
            continue;
        }
        let metric_definitions = metric_definitions.unwrap();
        metric_collector_manager.run(&metric_definitions).await;

        // Rerun the main application(controller)
        app.run().await;
    }
}
