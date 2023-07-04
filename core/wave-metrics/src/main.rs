use anyhow::{Ok, Result};
use args::Args;
use clap::Parser;
use data_layer::reader::wave_definition_reader::read_definition_yaml_file;
use log::error;
mod args;
mod metric_collector;

const DEFAULT_DEFINITION_FILE: &str = "./definition.yaml";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Parse command line arguments
    let args: Args = Args::parse();
    let definition = args
        .definition
        .unwrap_or(DEFAULT_DEFINITION_FILE.to_string());

    let result = read_definition_yaml_file(definition);
    if result.is_err() {
        error!("Error reading definition file: {}", result.err().unwrap());
        return Ok(());
    }
    let result = result.unwrap();

    // Process
    // 1. Read definition file and identify which kind of collector needed
    // 2. Download the collector binary if it doesn't exist
    // 3. Transform the metric definition into a collector configuration
    // 4. Run the collector

    let metric_collector =
        metric_collector::MetricCollector::new(result.metric_definitions, "./collectors.yaml");
    metric_collector.prepare_collector_binaries().await;

    Ok(())
}
