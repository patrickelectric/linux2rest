use crate::cli;
use crate::features;

use sinais::{Signal, SignalNoClone, _spawn};
use tokio::time::{sleep, Duration};
use tracing::*;

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
                        info!("{}: {:#?}", category, features::netstat::netstat());
                    }
                    cli::LogSetting::Platform => {
                        info!("{}: {:#?}", category, features::platform::platform());
                    }
                    cli::LogSetting::SerialPorts => {
                        info!("{}: {:#?}", category, features::serial::serial(None));
                    }
                    cli::LogSetting::SystemCpu => {
                        info!("{}: {:#?}", category, features::system::cpu());
                    }
                    cli::LogSetting::SystemDisk => {
                        info!("{}: {:#?}", category, features::system::disk());
                    }
                    cli::LogSetting::SystemInfo => {
                        info!("{}: {:#?}", category, features::system::info());
                    }
                    cli::LogSetting::SystemMemory => {
                        info!("{}: {:#?}", category, features::system::memory());
                    }
                    cli::LogSetting::SystemNetwork => {
                        info!("{}: {:#?}", category, features::system::network());
                    }
                    cli::LogSetting::SystemProcess => {
                        info!("{}: {:#?}", category, features::system::process());
                    }
                    cli::LogSetting::SystemTemperature => {
                        info!("{}: {:#?}", category, features::system::temperature());
                    }
                    cli::LogSetting::SystemUnixTimeSeconds => {
                        info!("{}: {:#?}", category, features::system::unix_time_seconds());
                    }
                }
            }

            counter += 1;
            sleep(Duration::from_secs(1)).await;
        }
    });
}
