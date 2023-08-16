/**
 * Main entry point for wave-metrics
 *
 * This program is responsible for:
 * 1. Reading the metric definitions from the database
 * 2. Downloading the collector binaries(Vector, Telegraf)
 * 3. Running the collectors and they will send the metrics to Wave API Server
 */
use anyhow::{Ok, Result};
use args::Args;
use clap::Parser;
use data_layer::data_layer::DataLayer;
use log::{debug, error};
use metric_collector_manager::MetricCollectorManager;
use utils::wave_config::WaveConfig;
mod args;
mod metric_collector_manager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Parse command line arguments
    let args: Args = Args::parse();

    // Read arguments
    let definition = args.definition.clone().unwrap_or_default();
    let config = args.config.clone().unwrap_or_default();
    let collectors_info = args.collectors_info.clone().unwrap_or_default();
    let watch_duration = args.watch_duration;

    let wave_config = WaveConfig::new(config.as_str());
    let db_url = wave_config.common.db_url;

    // TCP_SOCKET_ADDRESS from config file
    let output_url = wave_config.wave_metrics.output.url;

    // Create DataLayer
    let data_layer = DataLayer::new(db_url.as_str()).await;
    if !args.from_cli {
        data_layer.sync(definition.as_str()).await;
    }
    // Process
    // 1. Read definition file and identify which kind of collector needed
    // 2. Download the collector binary if it doesn't exist
    // 3. Transform the metric definition into a collector configuration
    // 4. Run the collector

    let metric_collector_manager = MetricCollectorManager::new(&collectors_info, &output_url);

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
