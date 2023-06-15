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

    /// Read initial plans from a yaml file. This plans will be saved into the database. If not specified, the default plans path './plan.yaml' will be used.
    #[arg(short, long)]
    pub plan: Option<String>,

    /// Watch the plan file for changes and reload accodingly
    #[arg(long)]
    pub watch_plan: bool,

    /// Do not run Wave API Server
    #[arg(long)]
    pub except_api_server: bool,

    /// Run Wave Web App(Web Interface)
    #[arg(long)]
    pub run_web_app: bool,
}
