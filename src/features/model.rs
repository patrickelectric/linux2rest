use paperclip::actix::Apiv2Schema;
use serde::Serialize;
use std::fs;
use std::process::Command;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct HardwareModel {
    model: String,
    arch: String,
    cpu_name: String,
}

impl HardwareModel {
    pub fn new() -> Self {
        Self {
            model: Self::get_model(),
            arch: Self::get_arch(),
            cpu_name: Self::get_cpu_name(),
        }
    }

    fn get_model() -> String {
        if let Ok(model) = fs::read_to_string("/proc/device-tree/model") {
            return model.trim().trim_matches(char::from(0)).to_string();
        }

        fs::read_to_string("/sys/devices/virtual/dmi/id/product_name")
            .unwrap_or_else(|_| "Unknown".to_string())
            .trim()
            .trim_matches(char::from(0))
            .to_string()
    }

    fn get_arch() -> String {
        String::from_utf8(match Command::new("uname").arg("-m").output() {
            Ok(output) => output.stdout,
            Err(_) => return "Unknown".to_string(),
        })
        .unwrap()
        .trim()
        .trim_matches(char::from(0))
        .to_string()
    }

    fn get_cpu_name() -> String {
        let stdout = String::from_utf8(match Command::new("lscpu").output() {
            Ok(output) => output.stdout,
            Err(_) => return "Unknown".to_string(),
        })
        .unwrap_or_else(|_| "Unknown".to_string());

        stdout
            .lines()
            .find(|line| line.starts_with("Model name:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().trim_matches(char::from(0)).to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }
}
