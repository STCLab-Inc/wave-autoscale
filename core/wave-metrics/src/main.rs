use anyhow::{Ok, Result};
use args::Args;
use clap::Parser;
use data_layer::{data_layer::DataLayer, reader::wave_config_reader::parse_wave_config_file};
use log::{debug, error};
use metric_collector_manager::MetricCollectorManager;
mod args;
mod metric_collector_manager;

const COLLECTORS_CONFIG_PATH: &str = "./collectors.yaml";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Parse command line arguments
    let args: Args = Args::parse();

    // Read arguments
    let definition = args.definition.clone().unwrap_or_default();
    let config = args.config.clone().unwrap_or_default();
    let watch_duration = args.watch_duration;

    // Read config file
    let config_result = parse_wave_config_file(config.as_str());

    // DB_URL from config file
    let db_url = config_result
        .get("COMMON")
        .and_then(|common| common.get("DB_URL"))
        .and_then(|db_url| db_url.as_str())
        .unwrap_or_default();

    // TCP_SOCKET_ADDRESS from config file
    let tcp_socket_address = config_result
        .get("WAVE-METRICS")
        .and_then(|wave_metrics| wave_metrics.get("OUTPUT"))
        .and_then(|output| output.get("TCP_SOCKET_ADDRESS"))
        .and_then(|tcp_socket_address| tcp_socket_address.as_str())
        .unwrap_or_default();

    // Create DataLayer
    let data_layer = DataLayer::new(db_url, definition.as_str()).await;

    // Process
    // 1. Read definition file and identify which kind of collector needed
    // 2. Download the collector binary if it doesn't exist
    // 3. Transform the metric definition into a collector configuration
    // 4. Run the collector

    let metric_collector_manager =
        MetricCollectorManager::new(COLLECTORS_CONFIG_PATH, tcp_socket_address);

    let mut watch_receiver = data_layer.watch(watch_duration);
    // Run this loop at once and then wait for changes
    let mut once = false;
    while !once || watch_receiver.changed().await.is_ok() {
        if once {
            let change = watch_receiver.borrow();
            debug!("DataLayer changed: {:?}", change);
        } else {
            once = true;
        }
        let metric_definitions = data_layer.get_all_metrics().await;
        if metric_definitions.is_err() {
            error!("Failed to get metric definitions");
            continue;
        }
        let metric_definitions = metric_definitions.unwrap();
        metric_collector_manager.run(&metric_definitions).await;
    }

    Ok(())
}
