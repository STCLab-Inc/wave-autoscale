use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TargetUrlDefinition {
    macos_x86_64: String,
    macos_aarch64: String,
    linux_x86_64: String,
    linux_aarch64: String,
    windows_x86_64: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CollectorDefinition {
    vector: TargetUrlDefinition,
    telegraf: TargetUrlDefinition,
}

impl CollectorDefinition {
    pub fn get_download_url(&self, name: &str) -> &str {
        match name {
            "vector_macos_x64_64" => self.vector.macos_x86_64.as_str(),
            "vector_macos_aarch64" => self.vector.macos_aarch64.as_str(),
            "vector_linux_x86_64" => self.vector.linux_x86_64.as_str(),
            "vector_linux_aarch64" => self.vector.linux_aarch64.as_str(),
            "vector_windows_x86_64" => self.vector.windows_x86_64.as_str(),
            "telegraf_macos_x64_64" => self.telegraf.macos_x86_64.as_str(),
            "telegraf_macos_aarch64" => self.telegraf.macos_aarch64.as_str(),
            "telegraf_linux_x86_64" => self.telegraf.linux_x86_64.as_str(),
            "telegraf_linux_aarch64" => self.telegraf.linux_aarch64.as_str(),
            "telegraf_windows_x86_64" => self.telegraf.windows_x86_64.as_str(),
            _ => "",
        }
    }
}
