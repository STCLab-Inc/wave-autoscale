pub mod process;
use data_layer::MetricDefinition;
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use std::cmp::min;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use tar::Archive;
use tracing::{debug, error, info};
use utils::wave_config::WaveConfig;
pub struct MetricCollectorManager {
    wave_config: WaveConfig,
    output_url: String,
    collector_log: bool,
    running_apps: Option<HashMap<String, std::process::Child>>,
}

impl MetricCollectorManager {
    pub fn new(
        wave_config: WaveConfig,
        output_url: &str,
        collector_log: bool,
        running_apps: Option<HashMap<String, std::process::Child>>,
    ) -> Self {
        Self {
            wave_config,
            output_url: output_url.to_string(),
            collector_log,
            running_apps,
        }
    }

    // Get the info of the OS and architecture - e.g. linux_x86_64
    fn get_os_arch(&self) -> String {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        format!("{}_{}", os, arch)
    }

    async fn download_file(url: &str, path: &str) -> Result<(), String> {
        // Reqwest setup
        let client = reqwest::Client::new();

        let res = client.get(url).send().await;

        if res.is_err() {
            return Err(format!(
                "Failed to GET from '{}', {:?}",
                &url,
                res.err().unwrap()
            ));
        }
        let res = res.unwrap();
        // get total size
        let total_size = res
            .content_length()
            .ok_or(format!("Failed to get content length from '{}'", &url))?;

        // TODO: use indicatif crate to show progress bar
        // Indicatif setup
        // let pb = ProgressBar::new(total_size);
        // pb.set_style(ProgressStyle::default_bar()
        //     .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        //     .progress_chars("#>-"));
        // pb.set_message(&format!("Downloading {}", url));

        // download chunks
        let mut file = File::create(path).or(Err(format!("Failed to create file '{}'", path)))?;
        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item.or(Err("Error while downloading file".to_string()))?;
            file.write_all(&chunk)
                .or(Err("Error while writing to file".to_string()))?;
            let new = min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            // pb.set_position(new);
        }
        // pb.finish_with_message(&format!("Downloaded {} to {}", url, path));
        Ok(())
    }

    fn decompress_file(source: &str, target: &str) -> Result<(), String> {
        let tar_gz = File::open(source);
        if tar_gz.is_err() {
            return Err(format!("Failed to open file '{}'", source));
        }
        let tar_gz = tar_gz.unwrap();
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        let unpacked = archive.unpack(target);
        if unpacked.is_err() {
            return Err(format!("Failed to unpack file '{}'", source));
        }

        Ok(())
    }

    // Download the collector binary if it doesn't exist
    pub async fn prepare_collector_binaries(&self, metric_definitions: &Vec<MetricDefinition>) {
        // Get the collector in MetricDefinition uniquely
        let mut collector_names: Vec<String> = Vec::new();
        for metric_definition in metric_definitions {
            if !collector_names.contains(&metric_definition.collector) {
                collector_names.push(metric_definition.collector.clone());
            }
        }
        // Download the collector binary if it doesn't exist
        for collector_name in collector_names {
            // Example: linux_x86_64
            let os_arch = self.get_os_arch();
            // Example: vector_linux_x86_64
            let collector_os_arch = format!("{}_{}", collector_name, os_arch);
            // Example: ./vector_linux_x86_64/vector
            let collector_binary_path = format!("./{}/{}", collector_os_arch, collector_name);

            // Check if the binary exists
            let path = std::path::Path::new(&collector_binary_path);
            if path.exists() {
                debug!("{} exists", collector_binary_path);
                continue;
            }

            // Create the temp directory if it doesn't exist
            let temp = "./temp".to_string();
            if std::fs::create_dir_all(&temp).is_err() {
                error!("Failed to create temp directory");
                continue;
            }

            // Download the file

            // Example: https://github.com/vectordotdev/vector/releases/download/v0.30.0/vector-0.30.0-x86_64-pc-windows-msvc.zip
            let download_url = self.wave_config.get_download_url(&collector_os_arch);

            // Example: vector-0.30.0-x86_64-pc-windows-msvc.zip
            let download_filename = download_url.split('/').last().unwrap();

            // Example: ./temp/vector-0.30.0-x86_64-pc-windows-msvc.zip
            let download_path = format!("{}/{}", temp, download_filename);

            // Download the file
            info!(
                "[metric-collector-manager] Downloading {} from {}",
                download_path, download_url
            );
            let result = Self::download_file(download_url, &download_path).await;
            if result.is_err() {
                error!("Error downloading file: {}", result.err().unwrap());
                continue;
            }
            info!(
                "[metric-collector-manager] Completed downloading {}",
                download_path
            );

            // Decompress the file
            // Example: ./temp/vector_linux_x86_64
            let decompress_path = format!("{}/{}", temp, collector_os_arch);
            let decompress_result = Self::decompress_file(&download_path, &decompress_path);
            if decompress_result.is_err() {
                error!(
                    "Error decompressing file: {}",
                    decompress_result.err().unwrap()
                );
                continue;
            }
            info!(
                "[metric-collector-manager] Decompressed {} to {}",
                download_path, collector_binary_path
            );

            // Find the binary file (e.g. vector)
            let mut binary_path: Option<String> = None;
            let walker = walkdir::WalkDir::new(&decompress_path);
            for entry in walker {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_file() {
                    let path_str = path.to_str().unwrap();
                    if path_str.ends_with(&collector_name) {
                        binary_path = Some(path_str.to_string());
                        break;
                    }
                }
            }

            // Move the binary file
            if let Some(binary_path) = binary_path {
                // Create the directory if it doesn't exist
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                std::fs::rename(&binary_path, &collector_binary_path).unwrap();
            } else {
                error!("Failed to find binary file");
                continue;
            }

            // Remove the temp file. If it fails, it's not a big deal
            let _ = std::fs::remove_dir_all(&temp);
        }
    }

    pub fn save_metric_definitions_to_vector_config(
        &self,
        metric_definitions: &Vec<&MetricDefinition>,
        config_path: &str,
    ) -> anyhow::Result<()> {
        /*
         * Example:
         *
         * [sources.metric_id_1]
         * type = "host_metrics"
         * scrape_interval_secs = 60
         *
         * [sinks.output_metric_id_1]
         * type = "http"
         * inputs = ["metric_id_1"]
         * compression = "gzip"
         * method = "post"
         * uri = "http://localhost:8080/api/metrics-receiver?metric_id=metric_id_1&collector=vector"
         * payload_prefix = '{"metrics": ['
         * payload_suffix = ']}'
         *
         * [sources.metric_id_2]
         * type = "gauge"
         *
         * [sinks.output_metric_id_2]
         * ...
         *
         */
        let root_toml =
            convert_metric_definitions_to_vector_toml(metric_definitions, self.output_url.clone());

        if root_toml.is_empty() {
            error!("Failed to convert metric definitions to toml");
            return Err(anyhow::anyhow!(
                "Failed to convert metric definitions to toml"
            ));
        }
        // Write the string to a file
        let result = std::fs::write(config_path, root_toml);
        if result.is_err() {
            let error = result.err().unwrap();
            error!("Failed to write to file: {}", error);
            return Err(anyhow::anyhow!("Failed to write to file: {}", error));
        }
        Ok(())
    }

    fn save_metric_definitions_to_telegraf_config(
        &self,
        metric_definitions: &Vec<&MetricDefinition>,
        config_path: &str,
    ) -> anyhow::Result<()> {
        /*
         * Example:
         *
         * [agent]
         * interval = "1s"
         * round_interval = true
         * metric_batch_size = 1000
         * metric_buffer_limit = 10000
         * collection_jitter = "0s"
         * flush_interval = "1s"
         * flush_jitter = "0s"
         * precision = "0s"
         * debug = false
         *
         * [[outputs.http]]
         * namepass = ["metric_id_test"]
         * url = "http://127.0.0.1:3024/api/metrics-receiver?collector=telegraf&metric_id=metric_id_test"
         * method = "POST"
         * data_format = "json"
         *
         * [outputs.http.tagpass]
         * metric_id = ["cloudwatch_metrics"]
         *
         * [[inputs.cloudwatch]]
         * namepass = "metric_id_test"
         * region = "ap-northeast-3"
         * namespaces = ["AWS/EC2"]
         * interval = "1m"
         * period = "1m"
         * delay = "0s"
         *
         * [[inputs.cloudwatch.metrics]]
         * names = ["CPUUtilization"]
         *
         * [[inputs.cloudwatch.metrics.dimensions]]
         * name = "InstanceId"
         * value = "i-0b49b58f6c93acf75"
         *
         * [inputs.cloudwatch.tags]
         * metric_id = "cloudwatch_metrics"
         *
         */
        let root_toml = convert_metric_definitions_to_telegraf_toml(
            metric_definitions,
            self.output_url.clone(),
        );

        if root_toml.is_empty() {
            error!("Failed to convert metric definitions to toml");
            return Err(anyhow::anyhow!(
                "Failed to convert metric definitions to toml"
            ));
        }
        // Write the string to a file
        let result = std::fs::write(config_path, root_toml);
        if result.is_err() {
            let error = result.err().unwrap();
            error!("Failed to write to file: {}", error);
            return Err(anyhow::anyhow!("Failed to write to file: {}", error));
        }
        Ok(())
    }

    pub async fn run(&mut self, metric_definitions: &Vec<MetricDefinition>) {
        info!(
            "[metric-collector-manager] {} metric definitions",
            metric_definitions.len()
        );
        // TODO: Validate the attribute 'collector' in metric_definitions. Now only support Vector
        // Prepare the collector binaries
        self.prepare_collector_binaries(metric_definitions).await;

        let mut collector_processes: Vec<process::AppInfo> = Vec::new();

        // Find the metric definitions that use Vector collector
        let mut vector_metric_definitions: Vec<&MetricDefinition> = Vec::new();
        for metric_definition in metric_definitions {
            if metric_definition.collector == "vector" {
                vector_metric_definitions.push(metric_definition);
            }
        }
        if !vector_metric_definitions.is_empty() {
            // Save the metric definitions to Vector config
            let os_arch = self.get_os_arch();
            let vector_dir_path = format!("./vector_{}", os_arch);
            let vector_config_path = format!("{}/vector.toml", vector_dir_path);
            let save_result = self.save_metric_definitions_to_vector_config(
                &vector_metric_definitions,
                vector_config_path.as_str(),
            );

            if save_result.is_ok() {
                // Run the collector binaries
                let vector_app_info = process::AppInfo {
                    name: "vector".to_string(),
                    command: format!("{}/vector", vector_dir_path),
                    args: Some(vec!["--config-toml".to_string(), vector_config_path]),
                    envs: None,
                    output: self.collector_log,
                };
                collector_processes.push(vector_app_info);
            }
        }

        // Find the metric definitions that use Telegraf collector
        let mut telegraf_metric_definitions: Vec<&MetricDefinition> = Vec::new();
        for metric_definition in metric_definitions {
            if metric_definition.collector == "telegraf" {
                telegraf_metric_definitions.push(metric_definition);
            }
        }
        if !telegraf_metric_definitions.is_empty() {
            // Save the metric definitions to Telegraf config
            let os_arch = self.get_os_arch();
            let telegraf_dir_path = format!("./telegraf_{}", os_arch);
            let telegraf_config_path = format!("{}/telegraf.conf", telegraf_dir_path);
            let save_result = self.save_metric_definitions_to_telegraf_config(
                &telegraf_metric_definitions,
                telegraf_config_path.as_str(),
            );

            if save_result.is_ok() {
                // Run the collector binaries
                let telegraf_app_info = process::AppInfo {
                    name: "telegraf".to_string(),
                    command: format!("{}/telegraf", telegraf_dir_path),
                    args: Some(vec!["--config".to_string(), telegraf_config_path]),
                    envs: None,
                    output: self.collector_log,
                };
                collector_processes.push(telegraf_app_info);
            }
        }

        // kill agent process
        if let Some(running_apps) = &mut self.running_apps {
            running_apps
                .iter_mut()
                .for_each(|(name, &mut ref mut child)| {
                    if let Err(child_kill) = child.kill() {
                        // retry 3 times
                        for idx in 1..4 {
                            if child.kill().is_ok() {
                                break;
                            };
                            error!("Failed to kill {} - try {}, {:?}", name, idx, child_kill);
                            if idx == 3 {
                                panic!("Failed to kill {}", name);
                            }
                        }
                    };
                    debug!("Killing {}", name);
                });
            // sleep 2 seconds
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        };

        if !collector_processes.is_empty() {
            // run agent process
            let running_apps = process::run_processes(&collector_processes);
            self.running_apps = Some(running_apps);
        }
    }
}

fn convert_metric_definitions_to_vector_toml(
    metric_definitions: &Vec<&MetricDefinition>,
    output_url: String,
) -> String {
    let mut root_toml = "".to_string();
    // Create a TOML array representing the metric definitions
    for metric_definition in metric_definitions {
        if !validate_vector_definition(metric_definition) {
            error!("[vector] Validation Failed");
            continue;
        }

        // metadata.sinks remove
        let mut metadata = metric_definition.metadata.clone();
        metadata.remove("sinks");

        // convert metric_definition.metadata to toml
        let Ok(metadata_toml) =
            serde_json::from_value::<toml::Value>(serde_json::json!(metadata)) else {
            error!("[vector] Failed to convert metadata to toml");
            continue;
        };

        let Some(sinks) = metric_definition.metadata.get("sinks") else {
            error!("[vector] sinks not found");
            continue;
        };

        let Some(sinks_object) = sinks.as_object() else {
            error!("[vector] Failed to convert metadata.sinks to as_object");
            continue;
        };

        // find sinks intput
        let mut sinks_input = Vec::<toml::Value>::new();
        sinks_object.keys().for_each(|key| {
            let Some(key_object) = sinks.get(key) else {
                error!("[vector] Failed to convert metadata.sinks to as_object for key: {}", key);
                return;
            };
            let Some(sinks_type_data) = key_object.get("type") else {
                error!("[vector] missing metadata.sinks type");
                return;
            };
            let Some(sinks_inputs_data) = key_object.get("inputs") else {
                error!("[vector] missing metadata.sinks inputs");
                return;
            };
            if sinks_type_data == "wave-autoscale" {
                let Some(sinks_inputs_data_arr) = sinks_inputs_data.as_array() else {
                    error!("[vector] Failed to convert metadata.sinks.inputs to as_array");
                    return;
                };
                for arr_data in sinks_inputs_data_arr {
                    let Some(input_str) = arr_data.as_str() else {
                        error!("[vector] sinks > wave-autoscale > input is not string");
                        return;
                    };
                    sinks_input.push(toml::Value::String(input_str.to_string()));
                }
            }
        });
        if sinks_input.is_empty() {
            error!("[vector] missing sinks > type: wave-autoscale > inputs data");
            continue;
        }

        // make new sinks
        let mut sinks_metric = toml::value::Table::new();
        sinks_metric.insert("type".to_string(), toml::Value::String("http".to_string()));
        sinks_metric.insert("inputs".to_string(), toml::Value::Array(sinks_input));
        sinks_metric.insert(
            "uri".to_string(),
            toml::Value::String(format!(
                "{}?metric_id={}&collector=vector",
                output_url, metric_definition.id
            )),
        );
        sinks_metric.insert(
            "method".to_string(),
            toml::Value::String("post".to_string()),
        );
        sinks_metric.insert(
            "compression".to_string(),
            toml::Value::String("gzip".to_string()),
        );
        sinks_metric.insert(
            "payload_prefix".to_string(),
            toml::Value::String("{\"metrics\": ".to_string()),
        );
        sinks_metric.insert(
            "payload_suffix".to_string(),
            toml::Value::String("}".to_string()),
        );
        let mut encoding = toml::value::Table::new();
        encoding.insert("codec".to_string(), toml::Value::String("json".to_string()));
        sinks_metric.insert("encoding".to_string(), toml::Value::Table(encoding));

        let sink_metric_id = format!("output_{}", metric_definition.id);
        let mut sinks_toml = toml::value::Table::new();
        sinks_toml.insert(sink_metric_id, toml::Value::Table(sinks_metric));
        let mut root_sinks_toml = toml::value::Table::new();
        root_sinks_toml.insert("sinks".to_string(), toml::Value::Table(sinks_toml));

        let Ok(metadata_toml_str) = toml::to_string(&metadata_toml) else {
            error!("[vector] Failed to convert metadata to toml string");
            continue;
        };
        let Ok(root_sinks_toml_str) = toml::to_string(&root_sinks_toml) else {
            error!("[vector] Failed to convert metadata.sinks to toml string");
            continue;
        };

        root_toml = root_toml + "\n" + &metadata_toml_str + "\n" + &root_sinks_toml_str + "\n";
    }

    debug!("Vector config:\n{}", root_toml);

    root_toml
}

fn convert_metric_definitions_to_telegraf_toml(
    metric_definitions: &Vec<&MetricDefinition>,
    output_url: String,
) -> String {
    let mut root_toml = "".to_string();
    // Create a TOML array representing the metric definitions
    for metric_definition in metric_definitions {
        if !validate_telegraf_definition(metric_definition) {
            error!("[telegraf] Validation Failed");
            continue;
        }

        // metadata.outputs, metadata.inputs remove
        let mut metadata = metric_definition.metadata.clone();
        metadata.remove("outputs");
        metadata.remove("inputs");

        // convert metric_definition.metadata to toml
        let Ok(metadata_toml) =
            serde_json::from_value::<toml::Value>(serde_json::json!(metadata)) else {
            error!("[telegraf] Failed to convert metadata to toml");
            continue;
        };

        let mut outputs_toml = toml::value::Array::new();
        let mut inputs_map =
            HashMap::<String, Vec<serde_json::Map<std::string::String, serde_json::Value>>>::new();

        // metadata.inputs
        let Some(inputs) = metric_definition.metadata.get("inputs").and_then(|inputs| inputs.as_object()) else {
            error!("[telegraf] Failed to convert metadata.inputs to as_object");
            continue;
        };

        // Example:
        // inputs: {             // inputs
        //   cloudwatch: [       // inputs_target
        //     {                 // inputs_target_item
        //        cloudwatch items
        //     },
        //     {
        //        cloudwatch items
        //     }
        //   ]
        // }
        for (inputs_key, inputs_target) in inputs.iter() {
            let Some(inputs_target) = inputs_target.as_array() else {
                error!("[telegraf] Failed to convert metadata.inputs to as_array");
                continue;
            };

            let mut transformed_inputs_target =
                Vec::<serde_json::Map<std::string::String, serde_json::Value>>::new();

            // insert required tags to inputs
            inputs_target.iter().for_each(|inputs_target_item| {
                let inputs_tags = inputs_target_item.get("tags");
                let mut inputs_tags_append = serde_json::Map::new();
                if let Some(inputs_tags) =
                    inputs_tags.and_then(|inputs_tags| inputs_tags.as_object())
                {
                    error!("[telegraf] Failed to convert metadata.inputs.tags to as_object");
                    inputs_tags_append = inputs_tags.clone();
                }
                inputs_tags_append.insert(
                    "metric_id".to_string(),
                    serde_json::json!(metric_definition.id.clone()),
                );
                let Some(inputs_target_item) = inputs_target_item.as_object() else {
                    error!("[telegraf] Failed to convert metadata.inputs array item to as_object");
                    return;
                };

                let mut tags_map = serde_json::Map::new();
                tags_map.insert("tags".to_string(), serde_json::json!(inputs_tags_append));
                let mut inputs_target_item_tags_append = inputs_target_item.clone();
                inputs_target_item_tags_append.append(&mut tags_map);
                transformed_inputs_target.push(inputs_target_item_tags_append);
            });

            inputs_map.insert(inputs_key.to_string(), transformed_inputs_target);
        }

        // metadata.outputs
        let Some(outputs) = metric_definition.metadata.get("outputs") else {
            error!("[telegraf] outputs not found");
            continue;
        };

        // find output waveautoscale
        if outputs.get("wave-autoscale").is_none() {
            error!("[telegraf] missing outputs > wave-autoscale data");
            continue;
        }

        // make new output
        let mut output_metric = toml::value::Table::new();
        output_metric.insert(
            "url".to_string(),
            toml::Value::String(format!(
                "{}?metric_id={}&collector=telegraf",
                output_url, metric_definition.id
            )),
        );
        output_metric.insert(
            "method".to_string(),
            toml::Value::String("POST".to_string()),
        );
        output_metric.insert(
            "data_format".to_string(),
            toml::Value::String("json".to_string()),
        );
        let Ok(required_tags) = serde_json::from_str::<toml::Value>(serde_json::json!({ "metric_id" : [metric_definition.id.to_string()] }).to_string().as_str()) else {
            continue;
        };
        output_metric.insert("tagpass".to_string(), required_tags);
        outputs_toml.push(toml::Value::Table(output_metric));

        // [[inputs]]
        let Ok(inputs_toml) =
            serde_json::from_value::<toml::Value>(serde_json::json!(inputs_map)) else {
            error!("[telegraf] Failed to convert inputs to toml");
            continue;
        };
        let mut root_inputs = toml::value::Table::new();
        root_inputs.insert("inputs".to_string(), inputs_toml);
        let Ok(inputs_toml_str) = toml::to_string(&root_inputs) else {
            error!("[telegraf] Failed to convert metadata to toml string");
            continue;
        };

        // [[outputs]]
        let mut outputs = toml::value::Table::new();
        outputs.insert("http".to_string(), toml::Value::Array(outputs_toml));
        let mut root_outputs = toml::value::Table::new();
        root_outputs.insert("outputs".to_string(), toml::Value::Table(outputs));
        let Ok(root_outputs_toml_str) = toml::to_string(&root_outputs) else {
            error!("[telegraf] Failed to convert metadata.output to toml string");
            continue;
        };

        // other metadata
        let Ok(metadata_toml_str) = toml::to_string(&metadata_toml) else {
            error!("[telegraf] Failed to convert metadata to toml string");
            continue;
        };

        root_toml = root_toml
            + "\n"
            + &metadata_toml_str
            + "\n"
            + &inputs_toml_str
            + "\n"
            + &root_outputs_toml_str
            + "\n";
    }

    // [agent]
    let Ok(agent_toml) = serde_json::from_value::<toml::Value>(serde_json::json!({
        "agent": {
            "interval": "1s",
            "round_interval": true,
            "metric_batch_size": 1000,
            "metric_buffer_limit": 10000,
            "collection_jitter": "0s",
            "flush_interval": "1s",
            "flush_jitter": "0s",
            "precision": "0s",
            "debug": false
        }
    })) else {
        error!("[telegraf] Failed to convert agent to toml");
        return String::new();
    };
    let Ok(agent_toml_str) = toml::to_string(&agent_toml) else {
        error!("[telegraf] Failed to convert agent to toml string");
        return String::new();
    };

    root_toml = root_toml + "\n" + &agent_toml_str + "\n";

    debug!("Telegraf config:\n{}", root_toml);

    root_toml
}

fn validate_vector_definition(metric_definitions: &MetricDefinition) -> bool {
    let metric_definition_json =
        serde_json::to_value::<&MetricDefinition>(metric_definitions).unwrap();

    // 1. check definition collector is "vector"
    if metric_definitions.collector != "vector" {
        return false;
    }

    // 2. check sinks type 'wave-autoscale'
    let Ok(sinks_type_path) = serde_json_path::JsonPath::parse("$.metadata.sinks.*.type") else {
        error!("[vector] no path - $.metadata.sinks.*.type");
        return false;
    };
    let Ok(sinks_type) = sinks_type_path
        .query(&metric_definition_json)
        .exactly_one() else {
        error!("[vector] sinks type not found");
        return false;
    };
    if sinks_type != &serde_json::json!("wave-autoscale".to_string()) {
        error!("[vector] sinks type is not 'wave-autoscale'");
        return false;
    }

    // 3. sinks inputs validation
    let mut sinks_inputs_target_ids = Vec::<String>::new();

    // 3-1. get sources ids
    let Ok(sources_ids_path) = serde_json_path::JsonPath::parse("$.metadata.sources") else {
        error!("[vector] no path - $.metadata.sources");
        return false;
    };
    let Ok(sources_ids) = sources_ids_path
        .query(&metric_definition_json)
        .exactly_one() else {
        error!("[vector] sources ids not found");
        return false;
    };
    let Some(sources_ids_object) = sources_ids.as_object() else {
        error!("[vector] Failed to convert sources ids to as_object");
        return false;
    };
    for sources_id in sources_ids_object.iter() {
        sinks_inputs_target_ids.push(sources_id.0.to_string());
    }

    // 3-2. get transforms ids
    let Ok(transtorms_ids_path) = serde_json_path::JsonPath::parse("$.metadata.transforms") else {
        error!("[vector] no path - $.metadata.transforms");
        return false;
    };
    let transtorms_ids = transtorms_ids_path
        .query(&metric_definition_json)
        .exactly_one();
    // transforms is optional
    if transtorms_ids.is_ok() {
        let Some(transtorms_ids_object) = transtorms_ids.unwrap().as_object() else {
            error!("[vector] Failed to convert transforms ids to as_object");
            return false;
        };
        for transtorms_id in transtorms_ids_object.iter() {
            sinks_inputs_target_ids.push(transtorms_id.0.to_string());
        }
    }

    // 3-3. check sinks inputs
    let Ok(sinks_inputs_path) = serde_json_path::JsonPath::parse("$.metadata.sinks.*.inputs") else {
        error!("[vector] no path - $.metadata.sinks.*.inputs");
        return false;
    };
    let Ok(sinks_inputs) = sinks_inputs_path
        .query(&metric_definition_json)
        .exactly_one() else {
        error!("[vector] sinks inputs not found");
        return false;
    };
    let mut is_sinks_inputs_target_ids = false;
    for sinks_input in sinks_inputs.as_array().unwrap() {
        let Some(sinks_input_str) = sinks_input.as_str() else {
            error!("[vector] sinks inputs is not string");
            return false;
        };
        if sinks_inputs_target_ids.contains(&sinks_input_str.to_string()) {
            is_sinks_inputs_target_ids = true;
        }
    }

    is_sinks_inputs_target_ids
}

fn validate_telegraf_definition(metric_definitions: &MetricDefinition) -> bool {
    let metric_definition_json =
        serde_json::to_value::<&MetricDefinition>(metric_definitions).unwrap();

    // 1. check definition collector is "telegraf"
    if metric_definitions.collector != "telegraf" {
        return false;
    }

    // 1. check outputs item contain 'wave-autoscale'
    let Ok(outputs_items_path) = serde_json_path::JsonPath::parse("$.metadata.outputs") else {
        error!("[telegraf] no path - $.metadata.outputs.wave-autoscale");
        return false;
    };
    let Ok(outputs_items) = outputs_items_path
        .query(&metric_definition_json)
        .exactly_one() else {
        error!("[telegraf] outputs.wave-autoscale not found");
        return false;
    };
    let Some(outputs_items_object) = outputs_items.as_object() else {
        error!("[telegraf] Failed to convert metadata.outputs to as_object");
        return false;
    };

    outputs_items_object.contains_key("wave-autoscale")
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_layer::types::object_kind::ObjectKind;
    use std::collections::HashMap;
    use tracing_test::traced_test;

    fn get_metric_collector_manager() -> MetricCollectorManager {
        // Remove wave.db
        let _ = std::fs::remove_file("./wave.db");

        let wave_config = WaveConfig::new("./tests/config/wave-config.yaml");
        MetricCollectorManager::new(
            wave_config,
            "http://localhost:3024/api/metrics-receiver",
            true,
            None,
        )
    }

    // Test whether it fetchs the os and arch correctly
    #[test]
    fn test_get_os_arch() {
        let manager = get_metric_collector_manager();
        let os_arch = manager.get_os_arch();
        assert!(
            os_arch == "linux_x86_64"
                || os_arch == "linux_aarch64"
                || os_arch == "macos_x86_64"
                || os_arch == "macos_aarch64"
                || os_arch == "windows_x86_64"
                || os_arch == "windows_aarch64",
            "os_arch should be linux or macos or windows",
        );
    }

    // Test whether it fetchs Vector binaries correctly for
    #[tokio::test]
    #[traced_test]
    async fn test_prepare_collector_binaries_vector() {
        let manager = get_metric_collector_manager();
        let metric_definitions = vec![
            // Vector
            MetricDefinition {
                id: "metric_id_1".to_string(),
                db_id: "db_id_1".to_string(),
                kind: ObjectKind::Metric,
                collector: "vector".to_string(),
                metadata: HashMap::new(),
            },
        ];

        let os_arch = manager.get_os_arch();
        let vector_dir_path = format!("./vector_{}", os_arch);
        let vector_bin_path = format!("{}/vector", vector_dir_path);
        // Remove the existing vector directory
        if std::path::Path::new(&vector_dir_path).exists() {
            std::fs::remove_dir_all(&vector_dir_path).unwrap();
        }

        manager
            .prepare_collector_binaries(&metric_definitions)
            .await;

        // Check whether the vector binary exists
        let vector_bin = std::fs::read(vector_bin_path).unwrap();
        assert!(!vector_bin.is_empty(), "vector binary should not be empty");

        // Remove the vector directory for the next test
        if std::path::Path::new(&vector_dir_path).exists() {
            std::fs::remove_dir_all(&vector_dir_path).unwrap();
        }
    }

    // Test whether it fetchs Telegraf binaries correctly
    #[tokio::test]
    #[traced_test]
    async fn test_prepare_collector_binaries_telegraf() {
        let manager = get_metric_collector_manager();
        let metric_definitions = vec![
            // Telegraf
            MetricDefinition {
                id: "metric_id_1".to_string(),
                db_id: "db_id_1".to_string(),
                kind: ObjectKind::Metric,
                collector: "telegraf".to_string(),
                metadata: HashMap::new(),
            },
        ];

        let os_arch = manager.get_os_arch();
        let telegraf_dir_path = format!("./telegraf_{}", os_arch);
        let telegraf_bin_path = format!("{}/telegraf", telegraf_dir_path);
        // Remove the existing telegraf directory
        if std::path::Path::new(&telegraf_dir_path).exists() {
            std::fs::remove_dir_all(&telegraf_dir_path).unwrap();
        }

        manager
            .prepare_collector_binaries(&metric_definitions)
            .await;

        // Check whether the telegraf binary exists
        let telegraf_bin = std::fs::read(telegraf_bin_path).unwrap();
        assert!(
            !telegraf_bin.is_empty(),
            "telegraf binary should not be empty"
        );

        // Remove the telegraf directory for the next test
        // if std::path::Path::new(&telegraf_dir_path).exists() {
        //     std::fs::remove_dir_all(&telegraf_dir_path).unwrap();
        // }
    }

    // Test whether it saves metric definitions to Vector config correctly
    #[test]
    #[traced_test]
    fn test_save_metric_definitions_to_vector_config() {
        let manager = get_metric_collector_manager();

        let vector_metadata_1 = r#"
        sources:
          metric_id_1:
            type: http_client
            query:
              "query": ['rate(istio_request_duration_milliseconds_sum{destination_workload="node-server-dp",response_code="200",reporter="destination"}[1m])']
        sinks:
          metric_id_1:
            type: wave-autoscale
            inputs: ["metric_id_1"]
        "#;
        let vector_metadata_2 = r#"
        sources:
          metric_id_2:
            type: http_client
            query:
              "query": ['rate(istio_request_duration_milliseconds_sum{destination_workload="node-server-dp",response_code="200",reporter="destination"}[1m])']
        transforms:
          my_transforms_id_1:
            inputs: ["metric_id_1"]
            type: remap
        sinks:
          metric_id_2:
            type: wave-autoscale
            inputs: ["my_transforms_id_1"]
        "#;
        let vector_metadata_hashmap_1 =
            serde_yaml::from_str::<HashMap<String, serde_json::Value>>(vector_metadata_1).unwrap();
        let vector_metadata_hashmap_2 =
            serde_yaml::from_str::<HashMap<String, serde_json::Value>>(vector_metadata_2).unwrap();

        let metric_definitions = vec![
            // Vector
            MetricDefinition {
                id: "metric_id_1".to_string(),
                db_id: "db_id_1".to_string(),
                kind: ObjectKind::Metric,
                collector: "vector".to_string(),
                metadata: vector_metadata_hashmap_1,
            },
            // Telegraf
            MetricDefinition {
                id: "metric_id_2".to_string(),
                db_id: "db_id_2".to_string(),
                kind: ObjectKind::Metric,
                collector: "vector".to_string(),
                metadata: vector_metadata_hashmap_2,
            },
        ];

        let os_arch = manager.get_os_arch();
        let vector_dir_path = format!("./vector_{}", os_arch);
        let vector_config_path = format!("{}/vector.toml", vector_dir_path);
        // Remove the existing vector directory
        if std::path::Path::new(&vector_dir_path).exists() {
            let _ = std::fs::remove_dir_all(&vector_dir_path);
        }

        // Create the vector directory
        std::fs::create_dir_all(&vector_dir_path).unwrap();

        let metric_definitions: Vec<&MetricDefinition> = metric_definitions.iter().collect();
        manager.save_metric_definitions_to_vector_config(
            &metric_definitions,
            vector_config_path.as_str(),
        );

        // Check whether the vector config exists
        let vector_config = std::fs::read_to_string(vector_config_path).unwrap();
        assert!(
            !vector_config.is_empty(),
            "vector config should not be empty"
        );

        // Check whether the vector config contains the metric definitions
        assert!(
            vector_config.contains("[sources.metric_id_1]"),
            "vector config should contain metric_id_1"
        );
        assert!(
            vector_config.contains("[sources.metric_id_2]"),
            "vector config should contain metric_id_2"
        );
        assert!(
            vector_config.contains("[sinks.output_metric_id_1]"),
            "vector config should contain output_metric_id_1"
        );
        assert!(
            vector_config.contains("[sinks.output_metric_id_2]"),
            "vector config should contain output_metric_id_2"
        );

        // Remove the vector directory for the next test
        if std::path::Path::new(&vector_dir_path).exists() {
            std::fs::remove_dir_all(&vector_dir_path).unwrap();
        }
    }

    #[test]
    #[traced_test]
    fn test_save_metric_definitions_to_telegraf_config() {
        let manager = get_metric_collector_manager();

        // let mut metadata_example: HashMap<String, serde_json::Value> = HashMap::new();
        // metadata_example.insert("key".to_string(), json!("value"));
        // let nested_metadata_example = json!([{}]);
        // metadata_example.insert("nested_metadata".to_string(), nested_metadata_example);

        let telegraf_metadata = r#"
        inputs:
          mem:
            - tags:
                metric_id: prometheus_metrics
        outputs:
          wave-autoscale:
            tagpass:
              metric_id: prometheus_metrics
        "#;
        let telegraf_metadata_hashmap =
            serde_yaml::from_str::<HashMap<String, serde_json::Value>>(telegraf_metadata).unwrap();

        let metric_definitions = vec![
            MetricDefinition {
                id: "metric_id_1".to_string(),
                db_id: "db_id_1".to_string(),
                kind: ObjectKind::Metric,
                collector: "telegraf".to_string(),
                metadata: telegraf_metadata_hashmap.clone(),
            },
            MetricDefinition {
                id: "metric_id_2".to_string(),
                db_id: "db_id_2".to_string(),
                kind: ObjectKind::Metric,
                collector: "telegraf".to_string(),
                metadata: telegraf_metadata_hashmap,
            },
        ];

        let os_arch = manager.get_os_arch();
        let telegraf_dir_path = format!("./telegraf_{}", os_arch);
        let telegraf_config_path = format!("{}/telegraf.conf", telegraf_dir_path);
        // Remove the existing vector directory
        if std::path::Path::new(&telegraf_dir_path).exists() {
            let _ = std::fs::remove_dir_all(&telegraf_dir_path);
        }

        // Create the vector directory
        std::fs::create_dir_all(&telegraf_dir_path).unwrap();

        let metric_definitions: Vec<&MetricDefinition> = metric_definitions.iter().collect();
        manager.save_metric_definitions_to_telegraf_config(
            &metric_definitions,
            telegraf_config_path.as_str(),
        );

        // Check whether the vector config exists
        let telegraf_config = std::fs::read_to_string(telegraf_config_path).unwrap();
        assert!(
            !telegraf_config.is_empty(),
            "telegraf config should not be empty"
        );

        // Check whether the telegraf config contains the metric definitions
        assert!(
            telegraf_config.contains("[[inputs.mem]]"),
            "telegraf config should contain [[inputs.mem]]"
        );
        // assert!(
        //     telegraf_config.contains("[sources.metric_id_2]"),
        //     "telegraf config should contain metric_id_2"
        // );
        // assert!(
        //     telegraf_config.contains("[sinks.output_metric_id_1]"),
        //     "telegraf config should contain output_metric_id_1"
        // );
        // assert!(
        //     telegraf_config.contains("[sinks.output_metric_id_2]"),
        //     "telegraf config should contain output_metric_id_2"
        // );

        // Remove the vector directory for the next test
        // if std::path::Path::new(&telegraf_dir_path).exists() {
        //     std::fs::remove_dir_all(&telegraf_dir_path).unwrap();
        // }
    }

    #[test]
    #[traced_test]
    fn test_vector_yaml_to_toml_success() {
        let metric_yaml_1 = r#"
        kind: Metric
        id: istio_request_duration_milliseconds_sum_1m
        collector: vector
        metadata:
          sources:
            my_source_id_1:
              type: http_client
              query:
                "query": ['rate(istio_request_duration_milliseconds_sum{destination_workload="node-server-dp",response_code="200",reporter="destination"}[1m])']
          transforms:
            my_transforms_id_1:
              inputs: ["my_source_id_1"]
              type: remap
          sinks:
            my_sinks_id:
              type: wave-autoscale
              inputs: ["my_transforms_id_1"]
        "#;

        let metric_yaml_2 = r#"
        kind: Metric
        id: istio_request_duration_milliseconds_sum_2m
        collector: vector
        metadata:
          sources:
            my_source_id_2:
              type: http_client
              query:
                "query": ['rate(istio_request_duration_milliseconds_sum{destination_workload="node-server-dp",response_code="200",reporter="destination"}[2m])']
          sinks:
            my_sinks_id_2:
              type: wave-autoscale
              inputs: ["my_source_id_2"]
        "#;

        let metric_definition_1 = serde_yaml::from_str::<MetricDefinition>(metric_yaml_1).unwrap();
        let metric_definition_2 = serde_yaml::from_str::<MetricDefinition>(metric_yaml_2).unwrap();

        let root_toml = convert_metric_definitions_to_vector_toml(
            &vec![&metric_definition_1, &metric_definition_2],
            "output_url".to_string(),
        );

        println!("root_toml:\n{}", root_toml);
        assert!(root_toml.contains("[sources.my_source_id_1]"));
        assert!(root_toml.contains("[sources.my_source_id_1.query]"));
        assert!(root_toml.contains("[transforms.my_transforms_id_1]"));
        assert!(root_toml.contains("inputs = [\"my_transforms_id_1\"]"));
        assert!(root_toml.contains("inputs = [\"my_source_id_2\"]"));
        assert!(root_toml.contains("method = \"post\""));
    }

    #[test]
    #[traced_test]
    fn test_vector_yaml_to_toml_validation() {
        let metric_yaml_success_1 = r#"
        kind: Metric
        id: metric_id_1
        collector: vector
        metadata:
          sources:
            my_source_id_1:
              type: http_client
            my_source_id_2:
              type: http_client
          transforms:
            my_transforms_id_1:
              inputs: ["my_source_id_1"]
              type: remap
            my_transforms_id_2:
              inputs: ["my_transforms_id_1"]
              type: remap
          sinks:
            my_sinks_id:
              type: wave-autoscale
              inputs: ["my_transforms_id_2"]
        "#;
        let metric_definition_success_1 =
            serde_yaml::from_str::<MetricDefinition>(metric_yaml_success_1).unwrap();
        assert!(validate_vector_definition(&metric_definition_success_1));

        let metric_yaml_success_2 = r#"
        kind: Metric
        id: metric_id_1
        collector: vector
        metadata:
          sources:
            my_source_id_2:
              type: http_client
          sinks:
            my_sinks_id_2:
              type: wave-autoscale
              inputs: ["my_source_id_2"]
        "#;
        let metric_definition_success_2 =
            serde_yaml::from_str::<MetricDefinition>(metric_yaml_success_2).unwrap();
        assert!(validate_vector_definition(&metric_definition_success_2));

        let metric_yaml_fail_1 = r#"
        kind: Metric
        id: metric_id_1
        collector: vector
        metadata:
          sources:
            my_source_id_2:
              type: http_client
          sinks:
            my_sinks_id_2:
              type: http_client
              inputs: ["my_source_id_2"]
        "#;
        let metric_definition_fail_1 =
            serde_yaml::from_str::<MetricDefinition>(metric_yaml_fail_1).unwrap();
        assert!(!validate_vector_definition(&metric_definition_fail_1));

        let metric_yaml_fail_2 = r#"
        kind: Metric
        id: metric_id_1
        collector: vector
        metadata:
          sources:
            my_source_id_1:
              type: http_client
            my_source_id_2:
              type: http_client
          transforms:
            my_transforms_id_1:
              inputs: ["my_source_id_1"]
              type: remap
            my_transforms_id_2:
              inputs: ["my_transforms_id_1"]
              type: remap
          sinks:
            my_sinks_id:
              type: wave-autoscale
              inputs: ["my_transforms_id_3"]
        "#;
        let metric_definition_fail_2 =
            serde_yaml::from_str::<MetricDefinition>(metric_yaml_fail_2).unwrap();
        assert!(!validate_vector_definition(&metric_definition_fail_2));
    }

    #[test]
    #[traced_test]
    fn test_telegraf_yaml_to_toml() {
        let yaml = r#"
        kind: Metric
        id: prometheus_metrics
        collector: telegraf
        metadata:
          inputs:
            prometheus:
              - urls: ["http://localhost:9090/metrics"]
                period: "10s"
                delay: "10s"
                interval: "10s"
                namepass: ["process_cpu_seconds_*"]
                tags:
                  test: test_tag
          outputs:
            wave-autoscale:
              tagpass:
                metric_id: prometheus_metrics
          secretstores:
            http:
              - id: "secretstore"
                url: "http://localhost/secrets"
        "#;

        let metric_definition = serde_yaml::from_str::<MetricDefinition>(yaml).unwrap();

        let root_toml = convert_metric_definitions_to_telegraf_toml(
            &vec![&metric_definition],
            "output_url".to_string(),
        );

        debug!("root_toml:\n{}", root_toml);
        assert!(root_toml.contains("[[secretstores.http]]"));
        assert!(root_toml.contains("id = \"secretstore\""));

        assert!(root_toml.contains("flush_interval = \"1s\""));

        assert!(root_toml.contains("[[inputs.prometheus]]"));
        assert!(root_toml.contains("namepass = [\"process_cpu_seconds_*\"]"));
        assert!(root_toml.contains("[inputs.prometheus.tags]"));
        assert!(root_toml.contains("metric_id = \"prometheus_metrics\""));
        assert!(root_toml.contains("test = \"test_tag\""));

        assert!(root_toml.contains("[[outputs.http]]"));
        assert!(root_toml.contains("[outputs.http.tagpass]"));
        assert!(root_toml.contains("metric_id = \"prometheus_metrics\""));
    }

    #[test]
    #[traced_test]
    fn test_telegraf_yaml_to_toml_validation() {
        let metric_yaml_success_1 = r#"
        kind: Metric
        id: prometheus_metrics
        collector: telegraf
        metadata:
          inputs:
            prometheus:
              - urls: ["http://localhost:9090/metrics"]
                period: "10s"
                delay: "10s"
          outputs:
            wave-autoscale: {}
        "#;
        let metric_definition_success_1 =
            serde_yaml::from_str::<MetricDefinition>(metric_yaml_success_1).unwrap();
        assert!(validate_telegraf_definition(&metric_definition_success_1));

        let metric_yaml_fail_1 = r#"
        kind: Metric
        id: prometheus_metrics
        collector: telegraf
        metadata:
          inputs:
            prometheus:
              - urls: ["http://localhost:9090/metrics"]
                period: "10s"
                delay: "10s"
          outputs:
            http:
              - url: "http://localhost/"
                method: "POST"
        "#;
        let metric_definition_fail_1 =
            serde_yaml::from_str::<MetricDefinition>(metric_yaml_fail_1).unwrap();
        assert!(!validate_telegraf_definition(&metric_definition_fail_1));
    }

    #[test]
    fn retry_test() {
        fn return_err() -> Result<(), ()> {
            Err(())
        }
        fn return_ok_of_num(ok_num: i32) -> Result<(), ()> {
            if ok_num == 2 {
                return Ok(());
            }
            Err(())
        }
        let mut retry_count = 0;
        if let Err(_result) = return_err() {
            // retry 3 times
            for idx in 1..4 {
                retry_count += 1;
                if return_ok_of_num(idx).is_ok() {
                    break;
                };
                if idx == 3 {
                    panic!("Failed to kill");
                }
            }
        };
        assert_eq!(retry_count, 2);
    }
}
