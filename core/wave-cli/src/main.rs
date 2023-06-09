use std::{
    collections::HashMap,
    process::{Child, Command},
};

use crate::args::Args;
use anyhow::Result;
use clap::Parser;

mod args;

#[macro_use]
extern crate log;

struct App {
    name: String,
    command: String,
    args: Vec<String>,
}

fn run_app(app: &App) -> Child {
    Command::new(&app.command)
        .args(&app.args)
        .spawn()
        .expect("Failed to start the application.")
}

fn is_node_installed() -> bool {
    match Command::new("node").arg("--version").output() {
        // TODO: Check the version
        Ok(output) => output.status.success(),
        Err(_) => {
            // Failed to execute the command (e.g., "node" not found)
            false
        }
    }
}

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // let current_path = std::env::current_dir().unwrap();
    // println!("The current directory is {}", current_path.display());

    let mut apps: Vec<App> = Vec::new();

    // Parse command line arguments
    let _args = Args::parse();
    let except_api_server = _args.except_api_server;
    let run_web_app = _args.run_web_app;

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

    if run_web_app {
        let web_app_file = std::path::Path::new("./wave-web-app/server.js");
        if !web_app_file.exists() {
            error!("wave-web-app does not exist");
            std::process::exit(1);
        }
        if !is_node_installed() {
            error!("Node.js is not installed. web-app needs Node.js to run.");
            std::process::exit(1);
        }
    }

    let mut args_for_controller: Vec<String> = Vec::new();

    if let Some(plan) = _args.plan {
        args_for_controller.push("--plan".to_string());
        args_for_controller.push(plan);
    }

    if let Some(config) = _args.config {
        args_for_controller.push("--config".to_string());
        args_for_controller.push(config);
    }

    apps.push(App {
        name: "wave-controller".to_string(),
        command: "./wave-controller".to_string(),
        args: args_for_controller,
    });

    if !except_api_server {
        apps.push(App {
            name: "wave-api-server".to_string(),
            command: "./wave-api-server".to_string(),
            args: Vec::new(),
        });
    }

    if run_web_app {
        apps.push(App {
            name: "wave-web-app".to_string(),
            command: "node".to_string(),
            args: vec!["./wave-web-app/server.js".to_string()],
        });
    }

    let mut running_apps: HashMap<String, Child> = HashMap::new();

    loop {
        {
            for app in &apps {
                if !running_apps.contains_key(&app.name) {
                    info!("Starting {}", app.name);
                    let child = run_app(app);
                    running_apps.insert(app.name.clone(), child);
                }
            }
        }
        {
            let mut to_remove: Vec<String> = Vec::new();
            for (name, child) in &mut running_apps {
                if let Some(exit_status) = child.try_wait().unwrap() {
                    info!("{} has exited with status: {}", name, exit_status);
                    to_remove.push(name.clone());
                }
            }
            for name in to_remove {
                running_apps.remove(&name);
            }
        }

        if running_apps.is_empty() {
            info!("All applications have exited");
            break;
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}
