use crate::args::Args;
use anyhow::Result;
use clap::Parser;

mod args;

#[macro_use]
extern crate log;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    let current_path = std::env::current_dir().unwrap();
    println!("The current directory is {}", current_path.display());

    // Parse command line arguments
    let _args = Args::parse();
    let except_api_server = _args.except_api_server;

    // Check that wave-controller file and wave-api-server exist
    let wave_autoscale_file = std::path::Path::new("./wave-controller");
    if !wave_autoscale_file.exists() {
        error!("wave-controller binary does not exist");
        std::process::exit(1);
    }

    if !except_api_server {
        let api_server_file = std::path::Path::new("./wave-api-server");
        if !api_server_file.exists() {
            error!("wave-api-server binary does not exist");
            std::process::exit(1);
        }
    }

    let mut wave_autoscale_child = std::process::Command::new("./wave-controller")
        // .arg("--plans")
        // .arg("./plans.yaml")
        // .arg("--config")
        // .arg("./config.yaml")
        .spawn()
        .expect("Failed to start wave-controller");

    if !except_api_server {
        let mut api_server_child = std::process::Command::new("./wave-api-server")
            .spawn()
            .expect("Failed to start wave-api-server");

        let _api_server_child_result = api_server_child.wait();
    }
    let _wave_autoscale_child_result = wave_autoscale_child.wait();

    Ok(())
}
