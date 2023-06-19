use args::Args;
use clap::Parser;

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

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::init();

    // Parse command line arguments
    let args = Args::parse();
    app::run(&args).await;
}
