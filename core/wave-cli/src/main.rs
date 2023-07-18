use crate::args::Args;
use anyhow::Result;
use clap::Parser;
use data_layer::reader::wave_config_reader::parse_wave_config_file;
use log::{debug, error, info};
use notify::{Config, PollWatcher, RecursiveMode, Watcher};
use regex::Regex;
use std::{
    collections::HashMap,
    path::Path,
    process::{Child, Command},
    time::Duration,
};
mod args;

const DEFAULT_CONFIG_FILE: &str = "./wave-config.yaml";
const DEFAULT_PLAN_FILE: &str = "./plan.yaml";
const WAVE_CONTROLLER: &str = "wave-controller";
const WAVE_API_SERVER: &str = "wave-api-server";
const WAVE_WEB_APP: &str = "wave-web-app";
const MINIMUM_NODE_VERSION: u32 = 14;

struct App {
    name: String,
    command: String,
    args: Vec<String>,
    envs: Option<HashMap<String, String>>,
}

fn run_app(app: &App) -> Child {
    let mut command = Command::new(&app.command);
    let command = if !app.args.is_empty() {
        command.args(&app.args)
    } else {
        &mut command
    };
    let command = if let Some(envs) = &app.envs {
        command.envs(envs)
    } else {
        command
    };
    command.spawn().unwrap()
}

fn is_node_installed() -> bool {
    match Command::new("node").arg("--version").output() {
        Ok(output) => {
            let output = String::from_utf8(output.stdout).unwrap();
            info!("Node version: {}", output);
            let regex = Regex::new(r"v(\d+)\.\d+\.\d+").unwrap();
            if let Some(captured) = regex.captures(output.as_str()) {
                if let Some(major_version) = captured.get(1) {
                    info!("Node major version: {}", major_version.as_str());
                    if let Ok(major_version) = major_version.as_str().parse::<u32>() {
                        if major_version >= MINIMUM_NODE_VERSION {
                            return true;
                        }
                    }
                }
            }
            false
        }
        Err(_) => {
            // Failed to execute the command (e.g., "node" not found)
            false
        }
    }
}

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Applications to run from wave-cli
    let mut apps: Vec<App> = Vec::new();

    // Parse command line arguments
    let args: Args = Args::parse();
    let config = args.config;
    let watch_plan = args.watch_plan;
    let plan = args.plan;
    let except_api_server = args.except_api_server;
    let run_web_app = args.run_web_app;

    // Create a channel to receive the events.
    let (watcher_tx, watcher_rx) = std::sync::mpsc::channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.

    let watcher_config = Config::default()
        .with_compare_contents(true)
        .with_poll_interval(Duration::from_secs(1));
    let mut plan_file_watcher = PollWatcher::new(watcher_tx, watcher_config)?;

    // Check config file exists
    let config_file = match config {
        Some(config) => {
            let config_path = std::path::Path::new(&config);
            if !config_path.exists() {
                error!("{} does not exist", config);
                DEFAULT_CONFIG_FILE.to_string()
            } else {
                config
            }
        }
        None => DEFAULT_CONFIG_FILE.to_string(),
    };

    // Check plan file exists
    let plan_file = match plan {
        Some(plan) => {
            let plan_path = std::path::Path::new(&plan);
            if !plan_path.exists() {
                error!("{} does not exist", plan);
                DEFAULT_PLAN_FILE.to_string()
            } else {
                plan
            }
        }
        None => DEFAULT_PLAN_FILE.to_string(),
    };

    // Check bin files exist
    let wave_autoscale_path = format!("./{}", WAVE_CONTROLLER);
    let wave_autoscale_file = std::path::Path::new(wave_autoscale_path.as_str());
    if !wave_autoscale_file.exists() {
        error!("{} binary does not exist", WAVE_CONTROLLER);
        std::process::exit(1);
    }

    if !except_api_server {
        let api_server_path = format!("./{}", WAVE_API_SERVER);
        let api_server_file = std::path::Path::new(api_server_path.as_str());
        if !api_server_file.exists() {
            error!("{} binary does not exist", WAVE_API_SERVER);
            std::process::exit(1);
        }
    }

    if run_web_app {
        // let web_app_file = std::path::Path::new("./wave-web-app/server.js");
        let web_app_path = format!("./{}", WAVE_WEB_APP);
        let web_app_file = std::path::Path::new(web_app_path.as_str());
        if !web_app_file.exists() {
            error!("{} does not exist", WAVE_WEB_APP);
            std::process::exit(1);
        }
        if !is_node_installed() {
            error!(
                "{} needs Node.js to run. Minimum version is {}.",
                WAVE_WEB_APP, MINIMUM_NODE_VERSION
            );
            std::process::exit(1);
        }
    }

    let mut args_for_controller: Vec<String> = Vec::new();

    let config_file_for_controller = config_file.clone();
    if !config_file_for_controller.is_empty() {
        args_for_controller.push("--config".to_string());
        args_for_controller.push(config_file_for_controller);
    }

    if !plan_file.clone().is_empty() {
        args_for_controller.push("--plan".to_string());
        args_for_controller.push(plan_file.clone());

        // Watch plan file
        if watch_plan {
            plan_file_watcher.watch(Path::new(&plan_file), RecursiveMode::Recursive)?;
            info!("Watching plan file: {}", &plan_file);
        }
    }

    let wave_controller_command = format!("./{}", WAVE_CONTROLLER);
    apps.push(App {
        name: WAVE_CONTROLLER.to_string(),
        command: wave_controller_command,
        args: args_for_controller,
        envs: None,
    });

    let wave_api_server_command = format!("./{}", WAVE_API_SERVER);
    if !except_api_server {
        let mut envs: HashMap<String, String> = HashMap::new();
        let config_file_for_api_server = config_file.clone();
        if !config_file_for_api_server.is_empty() {
            let config = parse_wave_config_file(config_file_for_api_server.as_str());
            if let Some(common_config) = config.get("COMMON").and_then(|v| v.as_mapping()) {
                if let Some(db_url) = common_config.get("DB_URL").and_then(|v| v.as_str()) {
                    envs.insert("DATABASE_URL".to_string(), db_url.to_string());
                }
            }

            if let Some(web_app_config) = config.get("API_SERVER").and_then(|v| v.as_mapping()) {
                debug!("api_server_config: {:?}", web_app_config);
                if let Some(port) = web_app_config.get("PORT").and_then(|v| v.as_u64()) {
                    envs.insert("PORT".to_string(), port.to_string());
                }
                if let Some(host) = web_app_config.get("HOST").and_then(|v| v.as_str()) {
                    envs.insert("HOST".to_string(), host.to_string());
                }
            }
        }

        debug!("envs: {:?}", envs);

        apps.push(App {
            name: WAVE_API_SERVER.to_string(),
            command: wave_api_server_command,
            args: Vec::new(),
            envs: Some(envs),
        });
    }

    if run_web_app {
        let mut envs: HashMap<String, String> = HashMap::new();
        let config_file_for_web_app: String = config_file.clone();
        if !config_file_for_web_app.is_empty() {
            let config = parse_wave_config_file(config_file_for_web_app.as_str());
            if let Some(web_app_config) = config.get("WEB_APP").and_then(|v| v.as_mapping()) {
                debug!("web_app_config: {:?}", web_app_config);
                if let Some(port) = web_app_config.get("PORT").and_then(|v| v.as_u64()) {
                    envs.insert("PORT".to_string(), port.to_string());
                }
                if let Some(host) = web_app_config.get("HOST").and_then(|v| v.as_str()) {
                    envs.insert("HOSTNAME".to_string(), host.to_string());
                }
            }
        }

        debug!("envs: {:?}", envs);
        let args = vec![format!("./{}/server.js", WAVE_WEB_APP)];
        apps.push(App {
            name: WAVE_WEB_APP.to_string(),
            command: "node".to_string(),
            args,
            envs: Some(envs),
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
        if watcher_rx.try_recv().is_ok() {
            // TODO: event is not used
            info!("Plan file has changed");
            if let Some(child) = running_apps.get_mut(WAVE_CONTROLLER) {
                info!("Killing {}", WAVE_CONTROLLER);
                let result = child.kill();
                if let Err(e) = result {
                    error!("Error killing {}: {:?}", WAVE_CONTROLLER, e);
                } else {
                    info!("{} killed", WAVE_CONTROLLER);
                }
            }
        }
        {
            let mut to_remove: Vec<String> = Vec::new();
            for (name, child) in &mut running_apps {
                if let Some(exit_status) = child.try_wait().unwrap() {
                    info!("{} has exited with status: {}", name, exit_status);
                    to_remove.push(name.clone());
                } else {
                    info!("{} is still running", name);
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
