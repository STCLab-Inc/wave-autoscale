use anyhow::Result;
use log::info;
use std::{
    collections::HashMap,
    process::{Child, Command},
};

pub struct AppInfo {
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub envs: Option<HashMap<String, String>>,
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
    match command.spawn() {
        Ok(child) => Ok(child),
        Err(_) => Err(anyhow::anyhow!("Error spawning {}", app_info.name)),
    }
}

pub fn run_processes(app_info_list: &Vec<AppInfo>) {
    let mut running_apps: HashMap<String, Child> = HashMap::new();
    loop {
        for app_info in app_info_list {
            if !running_apps.contains_key(&app_info.name) {
                info!("Starting {}", app_info.name);
                let child = spawn(app_info);
                if let Ok(child) = child {
                    running_apps.insert(app_info.name.clone(), child);
                }
            }
        }
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
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
