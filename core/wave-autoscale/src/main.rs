/**
 * Wave Autoscale
 */
mod app;
mod args;
mod metric_adapter;
mod metric_store;
mod scaling_component;
mod scaling_planner;
mod util;

use args::Args;
use clap::Parser;

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::init();

    // Parse command line arguments
    // Separate function to allow for unit testing
    let args = Args::parse();

    // Run the main application
    app::run(&args).await;
}
