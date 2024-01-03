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
    /// Verbose mode, Overridden by `--quiet`
    #[arg(short, long, action)]
    pub verbose: bool,
    /// Quiet mode, Overrides `--verbose`
    #[arg(short, long, action)]
    pub quiet: bool,
}
