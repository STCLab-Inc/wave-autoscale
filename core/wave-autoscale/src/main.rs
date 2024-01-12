/**
 * Wave Autoscale
 */
mod app;
mod metric_collector_manager;
mod metric_updater;
mod scaling_component;
mod scaling_planner;
mod util;
mod web_app_runner;

use api_server::app::run_api_server;
use data_layer::data_layer::DataLayer;
use metric_collector_manager::MetricsCollectorManager;
use std::sync::Arc;
use tokio::sync::watch;
use tracing::{debug, error};
use utils::{config_path::find_file_in_wa, wave_config::WaveConfig};

const LOCAL_DEFINITION_FILE_NAME: &str = "wave-definition.yaml";

#[tokio::main]
async fn main() {
    //
    // Initialize some features (Ctrl+C Signal, Command Line Arguments, Logger)
    //

    // Handle Ctrl+C Signal
    let _ = ctrlc::set_handler(move || {
        std::process::exit(0);
    });

    // WALog
    let wa_log = util::log::WALog::new();

    // Configuration
    let wave_config = WaveConfig::new();

    // Initialize logger
    if wave_config.quiet {
        wa_log.set_quiet();
    } else if wave_config.debug {
        wa_log.set_debug();
    } else {
        wa_log.set_info();
    }

    debug!("[WaveConfig] Config file parsed: {:?}", wave_config);

    //
    // Initialize the application (DataLayer, MetricsCollectorManager, API Server, Web App, and App)
    //

    // DataLayer
    let db_url = wave_config.db_url.clone();
    let metric_buffer_size_kb = wave_config.metric_buffer_size_kb;
    let enable_metrics_log = wave_config.enable_metrics_log;
    let data_layer =
        DataLayer::new(db_url.as_str(), metric_buffer_size_kb, enable_metrics_log).await;
    // Do not need RwLock or Mutex because the DataLayer is read-only.
    let shared_data_layer = Arc::new(data_layer);

    // MetricsCollectorManager
    let output_url = format!(
        "http://{}:{}/api/metrics-receiver",
        wave_config.host, wave_config.port
    );
    let mut metric_collector_manager =
        MetricsCollectorManager::new(wave_config.clone(), &output_url);

    // Run API Server
    let shared_data_layer_for_api_server = shared_data_layer.clone();
    let wave_config_for_api_server = wave_config.clone();
    // https://stackoverflow.com/questions/62536566/how-can-i-create-a-tokio-runtime-inside-another-tokio-runtime-without-getting-th
    tokio::task::spawn_blocking(move || {
        let _ = run_api_server(wave_config_for_api_server, shared_data_layer_for_api_server);
    });

    // Run Web App
    if wave_config.web_ui {
        let host = wave_config.web_ui_host.clone();
        let port = wave_config.web_ui_port;
        let _web_app_handle = tokio::spawn(async move {
            let _ = web_app_runner::run_web_app(host.as_str(), port);
        });
    }

    // Run the main application(controller)
    let mut app = app::App::new(wave_config.clone(), shared_data_layer.clone()).await;

    //
    // Run some jobs (Autoscaling History Remover, Reset definitions on startup, Watch the definition file, and the main application(controller))
    //

    // Remove autoscaling history
    if !wave_config.autoscaling_history_retention.is_empty() {
        app.run_autoscaling_history_cron_job(wave_config.autoscaling_history_retention);
    }

    // Reset definitions on startup
    if wave_config.reset_definitions_on_startup {
        let _ = shared_data_layer.delete_all_metrics().await;
        let _ = shared_data_layer.delete_all_scaling_components().await;
        let _ = shared_data_layer.delete_all_plans().await;
    }

    // Sync the definition file if it exists
    match find_file_in_wa(LOCAL_DEFINITION_FILE_NAME) {
        Ok(file_path) => {
            let _ = shared_data_layer.sync(file_path.as_str()).await;
        }
        Err(error_message) => {
            error!("{}", error_message);
        }
    }

    // TODO: replace watching the local definition to watching Git repository for GitOps
    // Watch the definition file
    // TODO: When it supports GitOps, it should be changed to watch the Git repository.
    let watch_duration = wave_config.watch_definition_duration;
    let mut watch_receiver: Option<watch::Receiver<String>> = if watch_duration != 0 {
        Some(shared_data_layer.watch_definitions_in_db(watch_duration))
    } else {
        None
    };

    // Run the main application(controller) in a loop
    // If watch_duration is 0, run the main application(controller) only once
    while watch_receiver.is_some() && watch_receiver.as_mut().unwrap().changed().await.is_ok() {
        // Update metric collectors
        // TODO: MetricCollectorManager could be moved into the app(controller)
        let shared_data_layer = shared_data_layer.clone();
        let metric_definitions = shared_data_layer.get_enabled_metrics().await;
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
