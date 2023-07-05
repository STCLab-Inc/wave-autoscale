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
    pub definition: Option<String>,
    #[arg(short, long)]
    pub config: Option<String>,
    #[arg(short, long, default_value_t = 5)]
    pub watch_duration: u64,
    /// Autoscaling history retention. You can specify a duration like 1d, 2w, 3m, 4y, etc.
    #[arg(long)]
    pub autoscaling_history_retention: Option<String>,
}
