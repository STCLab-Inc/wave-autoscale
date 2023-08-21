/**
 * Wave Autoscale
 */
mod app;
mod args;
mod metric_updater;
mod scaling_component;
mod scaling_planner;
mod util;
use std::sync::Arc;

use args::Args;
use clap::Parser;
use data_layer::data_layer::DataLayer;
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
    let data_layer = DataLayer::new(db_url.as_str(), args.definition.as_str()).await;
    // Do not need RwLock or Mutex because the DataLayer is read-only.
    let shared_data_layer = Arc::new(data_layer);

    // Run the main application
    let mut app = app::App::new(wave_config, shared_data_layer).await;
    app.run_with_watching().await;
}
