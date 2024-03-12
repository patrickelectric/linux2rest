use crate::cli;
use crate::features;

use sinais::{Signal, SignalNoClone, _spawn};
use tokio::time::{sleep, Duration};

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
                        dbg!(features::netstat::netstat());
                    }
                    cli::LogSetting::Platform => {
                        dbg!(features::platform::platform());
                    }
                    cli::LogSetting::SerialPorts => {
                        dbg!(features::serial::serial(None));
                    }
                    cli::LogSetting::SystemCpu => {
                        dbg!(features::system::cpu());
                    }
                    cli::LogSetting::SystemDisk => {
                        dbg!(features::system::disk());
                    }
                    cli::LogSetting::SystemInfo => {
                        dbg!(features::system::info());
                    }
                    cli::LogSetting::SystemMemory => {
                        dbg!(features::system::memory());
                    }
                    cli::LogSetting::SystemNetwork => {
                        dbg!(features::system::network());
                    }
                    cli::LogSetting::SystemProcess => {
                        dbg!(features::system::process());
                    }
                    cli::LogSetting::SystemTemperature => {
                        dbg!(features::system::temperature());
                    }
                    cli::LogSetting::SystemUnixTimeSeconds => {
                        dbg!(features::system::unix_time_seconds());
                    }
                }
            }

            counter += 1;
            sleep(Duration::from_secs(1)).await;
        }
    });
}
