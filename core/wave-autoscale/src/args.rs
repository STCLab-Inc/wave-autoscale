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
    #[arg(short, long, default_value = "./wave-config.yaml")]
    pub config: String,
    /// Read definition from a yaml file. If not specified, the default definition path './definition.yaml' will be used.
    #[arg(short, long, default_value = "./definition.yaml")]
    pub definition: String,
}
