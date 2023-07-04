use self::collector_definition::CollectorDefinition;
use data_layer::MetricDefinition;
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use log::{error, info};
use std::cmp::min;
use std::fs::File;
use std::io::Write;
use tar::Archive;
mod collector_definition;

pub struct MetricCollector {
    metric_definitions: Vec<MetricDefinition>,
    collector_definition: CollectorDefinition,
}

impl MetricCollector {
    pub fn new(metric_definitions: Vec<MetricDefinition>, collectors_file: &str) -> Self {
        let file = std::fs::File::open(collectors_file).unwrap();
        let collector_definition: CollectorDefinition = serde_yaml::from_reader(file).unwrap();
        Self {
            metric_definitions,
            collector_definition,
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
            let chunk = item.or(Err(format!("Error while downloading file")))?;
            file.write_all(&chunk)
                .or(Err(format!("Error while writing to file")))?;
            let new = min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            // pb.set_position(new);
        }
        // pb.finish_with_message(&format!("Downloaded {} to {}", url, path));
        return Ok(());
    }
    // Download the collector binary if it doesn't exist
    pub async fn prepare_collector_binaries(&self) {
        // Get the collector in MetricDefinition uniquely
        let mut collector_names: Vec<String> = Vec::new();
        for metric_definition in &self.metric_definitions {
            if !collector_names.contains(&metric_definition.collector) {
                collector_names.push(metric_definition.collector.clone());
            }
        }
        // Download the collector binary if it doesn't exist
        for collector_name in collector_names {
            let os_arch = self.get_os_arch();
            let collector_os_arch = format!("{}_{}", collector_name, os_arch);
            let collector_binary_path = format!("./{}/{}", collector_os_arch, collector_name);
            let path = std::path::Path::new(&collector_binary_path);
            if !path.exists() {
                let temp = "./temp".to_string();
                // Create the temp directory if it doesn't exist
                if std::fs::create_dir_all(&temp).is_err() {
                    error!("Failed to create temp directory");
                    continue;
                }

                // Download the file
                let download_url = self
                    .collector_definition
                    .get_download_url(&collector_os_arch);
                let download_filename = download_url.split('/').last().unwrap();
                let download_path = format!("{}/{}", temp, download_filename);
                info!("Downloading {} from {}", download_path, download_url);
                let result = Self::download_file(download_url, &download_path).await;
                if result.is_err() {
                    error!("Error downloading file: {}", result.err().unwrap());
                    continue;
                }
                info!("Downloaded {} to {}", download_url, download_path);
                // decompress the file
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

                // Find the binary file
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

                // Remove the temp file
                std::fs::remove_dir_all(&temp);
            }

            // let target_file = format!("./{}/{}", collector_str, collector_str);
        }
    }
    pub fn run(&self) {
        println!("Running metric collector");
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
}
