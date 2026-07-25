use std::collections::HashMap;

use crate::cli;
use crate::features;
use crate::zenoh::{self as zenoh_mod, publisher::Publisher};

use serde::Serialize;
use sinais::_spawn;
use tokio::time::{sleep, Duration};
use tracing::*;

struct Publishers {
    kernel: Publisher,
    journal: Publisher,
    categories: HashMap<cli::LogSetting, Publisher>,
}

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
        let mut publishers: Option<Publishers> = None;
        let topic = |name: &str| format!("services/system_information/{name}");
        let encoding = zenoh::bytes::Encoding::APPLICATION_JSON;

        loop {
            sleep(Duration::from_secs(1)).await;

            if publishers.is_none() {
                let Some(session) = zenoh_mod::get() else {
                    error!("Zenoh session not found");
                    continue;
                };
                let mut category_pubs = HashMap::new();
                for category in categories.keys() {
                    let name = category.to_string().replace('-', "_");
                    category_pubs.insert(
                        category.clone(),
                        Publisher::declare(&session, topic(&name), encoding.clone()).await,
                    );
                }
                publishers = Some(Publishers {
                    kernel: Publisher::declare(&session, topic("kernel"), encoding.clone()).await,
                    journal: Publisher::declare(&session, topic("journal"), encoding.clone()).await,
                    categories: category_pubs,
                });
            }

            let publishers = publishers.as_ref().unwrap();

            while let Ok(Some(message)) = kernel_client.try_next() {
                info!("Sending kernel message to zenoh: {message}");
                publishers.kernel.put(message).await;
            }

            while let Ok(Some(message)) = journal_client.try_next() {
                info!("Sending journal message to zenoh: {message}");
                publishers.journal.put(message).await;
            }

            for (category, interval) in categories.iter() {
                if !counter.is_multiple_of(*interval) {
                    continue;
                }

                let Some(publisher) = publishers.categories.get(category) else {
                    continue;
                };

                let data = match category {
                    cli::LogSetting::Netstat => {
                        serde_json::to_string(&features::netstat::netstat()).unwrap()
                    }
                    cli::LogSetting::Platform => match features::platform::platform() {
                        Ok(platform) => serde_json::to_string(&platform).unwrap(),
                        Err(error) => {
                            warn!("Skipping platform zenoh publish: {error}");
                            continue;
                        }
                    },
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

                info!("Sending data to zenoh: {}: {data}", publisher.key_expr());
                publisher.put(data).await;
            }

            counter += 1;
        }
    });
}
