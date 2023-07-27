/**
 * Command line arguments
 *
 * This module defines the command line arguments for the wave-autoscale binary.
 *
 */
use clap::Parser;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Read configuration from a yaml file. If not specified, the default config path './wave-config.yaml' will be used.
    #[arg(short, long)]
    pub config: Option<String>,

    /// Read definition from a yaml file. If not specified, the default definition path './definition.yaml' will be used.
    #[arg(short, long)]
    pub definition: Option<String>,

    /// Watch the plan file for changes and reload accodingly
    #[arg(long)]
    pub watch_definition: bool,

    /// Read collectors info from a yaml file. If not specified, the default collectors info path './collectors.yaml' will be used.
    #[arg(long)]
    pub collectors_info: Option<String>,

    /// Run Wave Metrics
    #[arg(long)]
    pub run_metrics: bool,

    /// Run Wave API Server
    #[arg(long)]
    pub run_api_server: bool,

    /// Run Wave Web App(Web Interface)
    #[arg(long)]
    pub run_web_app: bool,
}
