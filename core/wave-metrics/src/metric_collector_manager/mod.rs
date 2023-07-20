use self::collector_definition::CollectorDefinition;
use data_layer::MetricDefinition;
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use log::{debug, error, info};
use std::fs::File;
use std::io::Write;
use std::{cmp::min, collections::HashMap};
use tar::Archive;
use utils::process::{run_processes, AppInfo};
mod collector_definition;
pub struct MetricCollectorManager {
    collector_definition: CollectorDefinition,
    output_url: String,
}

impl MetricCollectorManager {
    pub fn new(collectors_file: &str, output_url: &str) -> Self {
        let file = std::fs::File::open(collectors_file).unwrap();
        let collector_definition: CollectorDefinition = serde_yaml::from_reader(file).unwrap();
        Self {
            collector_definition,
            output_url: output_url.to_string(),
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

        let res = client
            .get(url)
            .send()
            .await
            .or(Err(format!("Failed to GET from '{}'", &url)))?;

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
                info!("{} exists", collector_binary_path);
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
            let download_url = self
                .collector_definition
                .get_download_url(&collector_os_arch);

            // Example: vector-0.30.0-x86_64-pc-windows-msvc.zip
            let download_filename = download_url.split('/').last().unwrap();

            // Example: ./temp/vector-0.30.0-x86_64-pc-windows-msvc.zip
            let download_path = format!("{}/{}", temp, download_filename);

            // Download the file
            info!("Downloading {} from {}", download_path, download_url);
            let result = Self::download_file(download_url, &download_path).await;
            if result.is_err() {
                error!("Error downloading file: {}", result.err().unwrap());
                continue;
            }
            info!("Downloaded {} to {}", download_url, download_path);

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
                "Decompressed {} to {}",
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
    ) {
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
        // Create a TOML value representing your data structure
        let mut root = toml::value::Table::new();
        let mut sources = toml::value::Table::new();
        let mut sinks = toml::value::Table::new();

        // Create a TOML array representing the metric definitions
        for metric_definition in metric_definitions {
            if metric_definition.collector != "vector" {
                continue;
            }

            // Create a source
            let mut sources_metric = toml::value::Table::new();
            sources_metric.insert(
                "type".to_string(),
                toml::Value::String(metric_definition.metric_kind.to_string()),
            );
            // Add metadata to the source
            for (key, value) in &metric_definition.metadata {
                let value = serde_json::from_str(value.to_string().as_str());
                if let Ok(value) = value {
                    sources_metric.insert(key.to_string(), value);
                }
            }
            sources.insert(
                metric_definition.id.clone(),
                toml::Value::Table(sources_metric),
            );

            // Create a sink
            let mut sinks_metric = toml::value::Table::new();
            sinks_metric.insert("type".to_string(), toml::Value::String("http".to_string()));
            sinks_metric.insert(
                "inputs".to_string(),
                toml::Value::Array(vec![toml::Value::String(metric_definition.id.clone())]),
            );
            sinks_metric.insert(
                "uri".to_string(),
                toml::Value::String(format!(
                    "{}?metric_id={}&collector=vector",
                    self.output_url, metric_definition.id
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
            sinks.insert(sink_metric_id, toml::Value::Table(sinks_metric));
        }
        root.insert("sources".to_string(), toml::Value::Table(sources));
        root.insert("sinks".to_string(), toml::Value::Table(sinks));

        // Serialize it to a TOML string
        let toml = toml::to_string(&root).unwrap();
        debug!("Vector config:\n{}", toml);

        // Write the string to a file
        std::fs::write(config_path, toml).unwrap();
    }

    fn save_metric_definitions_to_telegraf_config(
        &self,
        metric_definitions: &Vec<&MetricDefinition>,
        config_path: &str,
    ) {
        /*
         * Example:
         *
         * [global_tags]
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
         * quite = false
         *
         * [[outputs.http]]
         * namepass = ["metric_id_test"]
         * url = "http://127.0.0.1:3024/api/metrics-receiver?collector=telegraf&metric_id=metric_id_test"
         * method = "POST"
         * data_format = "json"
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
         */
        // Create a TOML value representing your data structure
        let mut root = toml::value::Table::new();

        // [global_tags]
        let global_tags = toml::value::Table::new();
        root.insert("global_tags".to_string(), toml::Value::Table(global_tags));

        // [agent]
        let mut agent = toml::value::Table::new();
        agent.insert(
            "interval".to_string(),
            toml::Value::String("1s".to_string()),
        );
        agent.insert("round_interval".to_string(), toml::Value::Boolean(true));
        agent.insert("metric_batch_size".to_string(), toml::Value::Integer(1000));
        agent.insert(
            "metric_buffer_limit".to_string(),
            toml::Value::Integer(10000),
        );
        agent.insert(
            "collection_jitter".to_string(),
            toml::Value::String("0s".to_string()),
        );
        agent.insert(
            "flush_interval".to_string(),
            toml::Value::String("1s".to_string()),
        );
        agent.insert(
            "flush_jitter".to_string(),
            toml::Value::String("0s".to_string()),
        );
        agent.insert(
            "precision".to_string(),
            toml::Value::String("0s".to_string()),
        );
        agent.insert("debug".to_string(), toml::Value::Boolean(false));
        agent.insert("quiet".to_string(), toml::Value::Boolean(false));
        root.insert("agent".to_string(), toml::Value::Table(agent));

        // [[inputs]]
        let mut inputs = toml::value::Table::new();
        let mut inputs_array: HashMap<String, toml::value::Array> = HashMap::new();

        // [[outputs.http]]
        let mut outputs_http = toml::value::Array::new();

        // Create a TOML array representing the metric definitions
        for metric_definition in metric_definitions {
            if metric_definition.collector != "telegraf" {
                continue;
            }

            // Create a input
            let mut input_metric = toml::value::Table::new();

            // namepass = "metric_id"
            input_metric.insert(
                "name_override".to_string(),
                metric_definition.id.clone().into(),
            );

            // Add metadata to the source
            for (key, value) in &metric_definition.metadata {
                let Ok(value) = serde_json::from_str::<toml::Value>(value.to_string().as_str()) else {
                    continue;
                };
                input_metric.insert(key.to_string(), value);
            }

            // e.g. metric_kind: cloudwatch
            let kind = metric_definition.metric_kind.to_string();
            // e.g. [[inputs.cloudwatch]]
            let inputs_array_metric = match inputs_array.get_mut(&kind) {
                Some(inputs_array_metric) => inputs_array_metric,
                None => {
                    let inputs_metric = toml::value::Array::new();
                    inputs_array.insert(kind.clone(), inputs_metric);
                    inputs_array.get_mut(&kind).unwrap()
                }
            };
            inputs_array_metric.push(input_metric.into());

            // Create a output
            let mut output_metric = toml::value::Table::new();
            output_metric.insert(
                "namepass".to_string(),
                toml::Value::Array(vec![metric_definition.id.clone().into()]),
            );
            output_metric.insert(
                "url".to_string(),
                toml::Value::String(format!(
                    "{}?metric_id={}&collector=telegraf",
                    self.output_url, metric_definition.id
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
            outputs_http.push(toml::Value::Table(output_metric));
        }

        // Add the inputs and outputs to the root
        for (kind, inputs_array_metric) in inputs_array {
            inputs.insert(kind, inputs_array_metric.into());
        }
        root.insert("inputs".to_string(), toml::Value::Table(inputs));

        // [[outputs]]
        let mut outputs = toml::value::Table::new();
        outputs.insert("http".to_string(), toml::Value::Array(outputs_http));
        root.insert("outputs".to_string(), toml::Value::Table(outputs));

        // Serialize it to a TOML string
        let toml = toml::to_string(&root).unwrap();
        debug!("Telegraf config:\n{}", toml);

        // Write the string to a file
        std::fs::write(config_path, toml).unwrap();
    }

    pub async fn run(&self, metric_definitions: &Vec<MetricDefinition>) {
        // TODO: Validate the attribute 'collector' in metric_definitions. Now only support Vector

        // Prepare the collector binaries
        self.prepare_collector_binaries(metric_definitions).await;

        // Find the metric definitions that use Vector collector
        let mut vector_metric_definitions: Vec<&MetricDefinition> = Vec::new();
        for metric_definition in metric_definitions {
            if metric_definition.collector == "vector" {
                vector_metric_definitions.push(metric_definition);
            }
        }

        // Save the metric definitions to Vector config
        let os_arch = self.get_os_arch();
        let vector_dir_path = format!("./vector_{}", os_arch);
        let vector_config_path = format!("{}/vector.toml", vector_dir_path);
        self.save_metric_definitions_to_vector_config(
            &vector_metric_definitions,
            vector_config_path.as_str(),
        );

        // Find the metric definitions that use Telegraf collector
        let mut telegraf_metric_definitions: Vec<&MetricDefinition> = Vec::new();
        for metric_definition in metric_definitions {
            if metric_definition.collector == "telegraf" {
                telegraf_metric_definitions.push(metric_definition);
            }
        }

        // Save the metric definitions to Telegraf config
        let os_arch = self.get_os_arch();
        let telegraf_dir_path = format!("./telegraf_{}", os_arch);
        let telegraf_config_path = format!("{}/telegraf.conf", telegraf_dir_path);
        self.save_metric_definitions_to_telegraf_config(
            &telegraf_metric_definitions,
            telegraf_config_path.as_str(),
        );

        // Run the collector binaries
        let mut collector_processes: Vec<AppInfo> = Vec::new();
        let vector_app_info = AppInfo {
            name: "vector".to_string(),
            command: format!("{}/vector", vector_dir_path),
            args: Some(vec!["--config-toml".to_string(), vector_config_path]),
            envs: None,
        };
        collector_processes.push(vector_app_info);

        let telegraf_app_info = AppInfo {
            name: "telegraf".to_string(),
            command: format!("{}/telegraf", telegraf_dir_path),
            args: Some(vec!["--config".to_string(), telegraf_config_path]),
            envs: None,
        };
        collector_processes.push(telegraf_app_info);

        run_processes(&collector_processes);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use data_layer::types::object_kind::ObjectKind;

    use super::*;

    fn get_metric_collector_manager() -> MetricCollectorManager {
        // Remove wave.db
        let _ = std::fs::remove_file("./wave.db");

        MetricCollectorManager::new(
            "./tests/yaml/collectors.yaml",
            "http://localhost:8081/api/metrics-receiver",
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
    async fn test_prepare_collector_binaries_vector() {
        let manager = get_metric_collector_manager();
        let metric_definitions = vec![
            // Vector
            MetricDefinition {
                id: "metric_id_1".to_string(),
                db_id: "db_id_1".to_string(),
                kind: ObjectKind::Metric,
                collector: "vector".to_string(),
                metric_kind: "gauge".to_string(),
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
    async fn test_prepare_collector_binaries_telegraf() {
        let manager = get_metric_collector_manager();
        let metric_definitions = vec![
            // Telegraf
            MetricDefinition {
                id: "metric_id_1".to_string(),
                db_id: "db_id_1".to_string(),
                kind: ObjectKind::Metric,
                collector: "telegraf".to_string(),
                metric_kind: "gauge".to_string(),
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
    fn test_save_metric_definitions_to_vector_config() {
        let manager = get_metric_collector_manager();
        let metric_definitions = vec![
            // Vector
            MetricDefinition {
                id: "metric_id_1".to_string(),
                db_id: "db_id_1".to_string(),
                kind: ObjectKind::Metric,
                collector: "vector".to_string(),
                metric_kind: "gauge".to_string(),
                metadata: HashMap::new(),
            },
            // Telegraf
            MetricDefinition {
                id: "metric_id_2".to_string(),
                db_id: "db_id_2".to_string(),
                kind: ObjectKind::Metric,
                collector: "telegraf".to_string(),
                metric_kind: "gauge".to_string(),
                metadata: HashMap::new(),
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
    fn test_save_metric_definitions_to_telegraf_config() {
        let manager = get_metric_collector_manager();

        // let mut metadata_example: HashMap<String, serde_json::Value> = HashMap::new();
        // metadata_example.insert("key".to_string(), json!("value"));
        // let nested_metadata_example = json!([{}]);
        // metadata_example.insert("nested_metadata".to_string(), nested_metadata_example);

        let metric_definitions = vec![
            MetricDefinition {
                id: "metric_id_1".to_string(),
                db_id: "db_id_1".to_string(),
                kind: ObjectKind::Metric,
                collector: "telegraf".to_string(),
                metric_kind: "gauge".to_string(),
                metadata: HashMap::new(),
            },
            MetricDefinition {
                id: "metric_id_2".to_string(),
                db_id: "db_id_2".to_string(),
                kind: ObjectKind::Metric,
                collector: "telegraf".to_string(),
                metric_kind: "mem".to_string(),
                metadata: HashMap::new(),
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
}
