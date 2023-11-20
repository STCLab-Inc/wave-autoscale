use anyhow::Result;
use std::{
    collections::HashMap,
    process::{Child, Command, Stdio},
};
use tracing::debug;

pub struct AppInfo {
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub envs: Option<HashMap<String, String>>,
    pub output: bool,
}

fn spawn(app_info: &AppInfo) -> Result<Child> {
    let mut command = Command::new(&app_info.command);
    // Arguments
    let command = if let Some(args) = &app_info.args {
        command.args(args)
    } else {
        &mut command
    };
    // Envs
    let command = if let Some(envs) = &app_info.envs {
        command.envs(envs)
    } else {
        command
    };
    // Output
    let command = if app_info.output {
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit())
    } else {
        command.stdout(Stdio::null()).stderr(Stdio::null())
    };
    match command.spawn() {
        Ok(child) => Ok(child),
        Err(_) => Err(anyhow::anyhow!("Error spawning {}", app_info.name)),
    }
}

pub fn run_processes(app_info_list: &Vec<AppInfo>) -> HashMap<String, Child> {
    let mut running_apps: HashMap<String, Child> = HashMap::new();
    for app_info in app_info_list {
        debug!(">> Starting {}", app_info.name);
        let child = spawn(app_info);
        if let Ok(child) = child {
            running_apps.insert(app_info.name.clone(), child);
        }
    }
    running_apps
}
