use regex::Regex;
use std::{
    collections::HashMap,
    process::{Child, Command},
};
use tracing::{debug, error};

const WAVE_WEB_APP: &str = "wave-autoscale-ui";
const MINIMUM_NODE_VERSION: u32 = 14;

struct App {
    command: String,
    args: Vec<String>,
    envs: Option<HashMap<String, String>>,
}

fn run_app(app: &App) -> std::io::Result<Child> {
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
    command.spawn()
}

fn is_node_installed() -> bool {
    match Command::new("node").arg("--version").output() {
        Ok(output) => {
            let Ok(output) = String::from_utf8(output.stdout) else {
                return false;
            };
            debug!("[web-app-runner] Node version: {}", output);
            let Ok(regex) = Regex::new(r"v(\d+)\.\d+\.\d+") else {
                return false;
            };
            let Some(captured) = regex.captures(output.as_str()) else {
                return false;
            };
            let Some(major_version) = captured.get(1) else {
                return false;
            };
            debug!(
                "[web-app-runner] Node major version: {}",
                major_version.as_str()
            );
            let Ok(major_version) = major_version.as_str().parse::<u32>() else {
                return false;
            };
            if major_version < MINIMUM_NODE_VERSION {
                return false;
            }
            true
        }
        Err(_) => {
            // Failed to execute the command (e.g., "node" not found)
            false
        }
    }
}

pub fn run_web_app(host: &str, port: u16) -> anyhow::Result<()> {
    let web_app_path = format!("./{}", WAVE_WEB_APP);
    let web_app_file = std::path::Path::new(web_app_path.as_str());
    if !web_app_file.exists() {
        error!("[web-app-runner] {} does not exist", WAVE_WEB_APP);
        return Err(anyhow::anyhow!("{} does not exist", WAVE_WEB_APP));
    }
    if !is_node_installed() {
        error!(
            "{} needs Node.js to run. Minimum version is {}.",
            WAVE_WEB_APP, MINIMUM_NODE_VERSION
        );
        std::process::exit(1);
    }

    let mut envs: HashMap<String, String> = HashMap::new();
    envs.insert("HOSTNAME".to_string(), host.to_string());
    envs.insert("PORT".to_string(), port.to_string());

    let args = vec![format!("./{}/server.js", WAVE_WEB_APP)];

    let result = run_app(&App {
        command: "node".to_string(),
        args,
        envs: Some(envs),
    });
    if result.is_err() {
        error!("Failed to run {}", WAVE_WEB_APP);
        return Err(anyhow::anyhow!("Failed to run {}", WAVE_WEB_APP));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_is_node_installed() {
        assert!(is_node_installed());
    }
}
