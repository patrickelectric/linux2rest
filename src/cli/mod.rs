use anyhow::*;
use std::collections::HashMap;
use std::sync::Arc;
use structopt::StructOpt;
use strum_macros::{Display, EnumString};

#[derive(Clone, Debug, Display, Hash, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum LogSetting {
    Netstat,
    Platform,
    SerialPorts,
    SystemCpu,
    SystemDisk,
    SystemInfo,
    SystemMemory,
    SystemNetwork,
    SystemProcess,
    SystemTemperature,
    SystemUnixTimeSeconds,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = env!("CARGO_PKG_NAME"),
    about = "Linux to REST API.",
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"))
]
pub struct Arguments {
    /// Activate debug/verbose mode
    #[structopt(short, long)]
    pub verbose: bool,

    /// Specifies the path in witch the logs will be stored.
    #[structopt(long, default_value = "./logs")]
    pub log_path: String,

    /// Port to be used for REST API server
    #[structopt(long, default_value = "6030")]
    pub port: u16,

    /// Set logging intervals for various services in a comma-separated list (e.g., "system-cpu=10,system-disk=30")
    /// Valid keys are: netstat, platform, serial-ports, system-cpu, system-disk, system-info, system-memory, system-network, system-process, system-temperature, system-unix-time-seconds
    #[structopt(long, parse(try_from_str = parse_log_settings), default_value="")]
    pub log_settings: HashMap<LogSetting, u64>,
}

lazy_static! {
    static ref ARGS: Arc<Arguments> = Arc::new(Arguments::from_args());
}

pub fn args() -> Arc<Arguments> {
    ARGS.clone()
}

pub fn command_line_string() -> String {
    std::env::args().collect::<Vec<String>>().join(" ")
}

fn parse_log_settings(s: &str) -> Result<HashMap<LogSetting, u64>> {
    let clean_string = s.trim();
    if clean_string.is_empty() {
        return Ok(HashMap::new());
    }

    let pairs = s.split(',');
    let mut settings = HashMap::new();

    for pair in pairs {
        let mut kv = pair.split('=');
        match (kv.next(), kv.next()) {
            (Some(key), Some(value)) => {
                let key: LogSetting = key.parse()?;
                let val: u64 = value
                    .parse()
                    .map_err(|_| anyhow!("Invalid value for '{key:?}', expected an integer"))?;
                validate_interval(&key, val)?;
                settings.insert(key, val);
            }
            _ => {
                return Err(anyhow!(
                    "Invalid format for setting '{pair}', expected format key=value"
                ))
            }
        }
    }

    Ok(settings)
}

fn validate_interval(key: &LogSetting, val: u64) -> Result<()> {
    match key {
        LogSetting::SystemUnixTimeSeconds if val < 1 => Err(anyhow!(
            "Interval for '{key:?}' must not be less than 1 second."
        )),
        LogSetting::SystemTemperature | LogSetting::Platform if val < 5 => Err(anyhow!(
            "Interval for '{key:?}' must not be less than 5 seconds."
        )),
        LogSetting::SystemProcess
        | LogSetting::Netstat
        | LogSetting::SerialPorts
        | LogSetting::SystemCpu
        | LogSetting::SystemMemory
        | LogSetting::SystemNetwork
            if val < 10 =>
        {
            Err(anyhow!(
                "Interval for '{key:?}' must not be less than 10 seconds."
            ))
        }
        LogSetting::SystemDisk if val < 30 => Err(anyhow!(
            "Interval for '{key:?}' must not be less than 30 seconds."
        )),
        _ => Ok(()),
    }
}
