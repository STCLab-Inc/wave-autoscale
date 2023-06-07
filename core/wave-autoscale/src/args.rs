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
    #[arg(short, long)]
    pub plans: Option<String>,
    #[arg(short, long)]
    pub config: Option<String>,
}
