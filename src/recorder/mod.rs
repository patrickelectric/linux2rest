use crate::cli;
use crate::features;
use crate::zenoh as zenoh_mod;

use serde::Serialize;
use sinais::_spawn;
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

    let mut kernel_client = features::kernel::ask_for_client();
    let mut journal_client = features::journal::ask_for_client();

    _spawn(module_path!().into(), async move {
        let mut counter: u64 = 0;

        let zenoh_topic_name = "system_information/{}";
        loop {
            sleep(Duration::from_secs(1)).await;

            let Some(zenoh_session) = zenoh_mod::get() else {
                error!("Zenoh session not found");
                continue;
            };

            while let Ok(Some(message)) = kernel_client.try_next() {
                info!("Sending kernel message to zenoh: {message}");
                zenoh_session
                    .put(zenoh_topic_name.replace("{}", "kernel"), message)
                    .encoding(zenoh::bytes::Encoding::APPLICATION_JSON)
                    .await
                    .unwrap();
            }

            while let Ok(Some(message)) = journal_client.try_next() {
                info!("Sending journal message to zenoh: {message}");
                zenoh_session
                    .put(zenoh_topic_name.replace("{}", "journal"), message)
                    .encoding(zenoh::bytes::Encoding::APPLICATION_JSON)
                    .await
                    .unwrap();
            }

            for (category, interval) in categories.iter() {
                if counter % interval != 0 {
                    continue;
                }

                let topic_name =
                    zenoh_topic_name.replace("{}", &category.to_string().replace("-", "_"));
                let data = match category {
                    cli::LogSetting::Netstat => {
                        serde_json::to_string(&features::netstat::netstat()).unwrap()
                    }
                    cli::LogSetting::Platform => {
                        serde_json::to_string(&features::platform::platform()).unwrap()
                    }
                    cli::LogSetting::SerialPorts => {
                        serde_json::to_string(&features::serial::serial(None)).unwrap()
                    }
                    cli::LogSetting::Cpu => {
                        serde_json::to_string(&features::system::cpu()).unwrap()
                    }
                    cli::LogSetting::Disk => {
                        serde_json::to_string(&features::system::disk()).unwrap()
                    }
                    cli::LogSetting::Info => {
                        serde_json::to_string(&features::system::info()).unwrap()
                    }
                    cli::LogSetting::Memory => {
                        serde_json::to_string(&features::system::memory()).unwrap()
                    }
                    cli::LogSetting::Network => {
                        serde_json::to_string(&features::system::network()).unwrap()
                    }
                    cli::LogSetting::Process => {
                        serde_json::to_string(&features::system::process()).unwrap()
                    }
                    cli::LogSetting::Temperature => {
                        serde_json::to_string(&features::system::temperature()).unwrap()
                    }
                    cli::LogSetting::UnixTimeSeconds => {
                        serde_json::to_string(&features::system::unix_time_seconds()).unwrap()
                    }
                    cli::LogSetting::Usb => {
                        serde_json::to_string(&features::usb::usb_devices()).unwrap()
                    }
                };

                info!("Sending data to zenoh: {topic_name}: {data}");

                zenoh_session
                    .put(topic_name, data)
                    .encoding(zenoh::bytes::Encoding::APPLICATION_JSON)
                    .await
                    .unwrap();
            }

            counter += 1;
        }
    });
}
