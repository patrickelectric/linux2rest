use crate::cli;
use crate::features;

use serde::Serialize;
use sinais::{Signal, SignalNoClone, _spawn};
use tokio::time::{sleep, Duration};
use tracing::*;

pub fn print<T: Serialize>(category: &cli::LogSetting, data: T) {
    let json = serde_json::to_string(&data).unwrap();
    info!("{category}: {json}");
}

pub fn start() {
    let categories = cli::args().as_ref().log_settings.clone();
    if categories.is_empty() {
        return;
    }

    _spawn(module_path!().into(), async move {
        let mut counter: u64 = 0;
        loop {
            for (category, interval) in categories.iter() {
                if counter % interval != 0 {
                    continue;
                }

                match category {
                    cli::LogSetting::Netstat => {
                        print(category, features::netstat::netstat());
                    }
                    cli::LogSetting::Platform => {
                        print(category, features::platform::platform());
                    }
                    cli::LogSetting::SerialPorts => {
                        print(category, features::serial::serial(None));
                    }
                    cli::LogSetting::SystemCpu => {
                        print(category, features::system::cpu());
                    }
                    cli::LogSetting::SystemDisk => {
                        print(category, features::system::disk());
                    }
                    cli::LogSetting::SystemInfo => {
                        print(category, features::system::info());
                    }
                    cli::LogSetting::SystemMemory => {
                        print(category, features::system::memory());
                    }
                    cli::LogSetting::SystemNetwork => {
                        print(category, features::system::network());
                    }
                    cli::LogSetting::SystemProcess => {
                        print(category, features::system::process());
                    }
                    cli::LogSetting::SystemTemperature => {
                        print(category, features::system::temperature());
                    }
                    cli::LogSetting::SystemUnixTimeSeconds => {
                        print(category, features::system::unix_time_seconds());
                    }
                }
            }

            counter += 1;
            sleep(Duration::from_secs(1)).await;
        }
    });
}
