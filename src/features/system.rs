use std::sync::{Arc, Mutex};
use sysinfo::CpuExt;
use sysinfo::PidExt;

use cached::proc_macro::cached;
use log::*;
use paperclip::actix::Apiv2Schema;
use pnet;
use serde::Serialize;
use sysinfo::{
    ComponentExt, DiskExt, NetworkExt, NetworksExt, ProcessExt, System as sysSystem, SystemExt,
};

lazy_static! {
    static ref SYSTEM: Arc<Mutex<sysSystem>> = Arc::new(Mutex::new(sysSystem::new()));
}

#[derive(Clone, Debug, Serialize, Apiv2Schema)]
pub struct Cpu {
    name: String,
    usage: f32,
    frequency: u64,
    vendor_id: String,
    brand: String,
}

#[derive(Clone, Debug, Serialize, Apiv2Schema)]
pub struct Disk {
    name: String,
    filesystem_type: String,
    #[serde(rename = "type")]
    disk_type: String,
    mount_point: String,
    available_space_B: u64,
    total_space_B: u64,
}

#[derive(Clone, Debug, Serialize, Apiv2Schema)]
pub struct OsInfo {
    system_name: String,
    kernel_version: String,
    os_version: String,
    host_name: String,
}

#[derive(Clone, Debug, Serialize, Apiv2Schema)]
pub struct MemoryUsage {
    used_kB: u64,
    total_kB: u64,
}

#[derive(Clone, Debug, Serialize, Apiv2Schema)]
pub struct Memory {
    ram: MemoryUsage,
    swap: MemoryUsage,
}

#[derive(Clone, Debug, Serialize, Apiv2Schema)]
pub struct Network {
    name: String,
    description: String,

    mac: String,
    ips: Vec<String>,

    is_up: bool,
    is_loopback: bool,

    received_B: u64,
    total_received_B: u64,

    transmitted_B: u64,
    total_transmitted_B: u64,

    packets_received: u64,
    total_packets_received: u64,

    packets_transmitted: u64,
    total_packets_transmitted: u64,

    errors_on_received: u64,
    total_errors_on_received: u64,

    errors_on_transmitted: u64,
    total_errors_on_transmitted: u64,
}

//TODO: be consistent between _B, _b and bytes
#[derive(Clone, Debug, Serialize, Apiv2Schema)]
pub struct DiskUsage {
    total_written_bytes: u64,
    written_bytes: u64,
    total_read_bytes: u64,
    read_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Apiv2Schema)]
pub struct Process {
    name: String,
    pid: u32,
    status: String,
    command: Vec<String>,
    executable_path: String,
    environment: Vec<String>,
    working_directory: String,
    root_directory: String,
    used_memory_kB: u64,
    virtual_memory_kB: u64,
    parent_process: Option<u32>,
    running_time: u64,
    cpu_usage: f32,
    disk_usage: DiskUsage,
}

#[derive(Clone, Debug, Serialize, Apiv2Schema)]
pub struct Temperature {
    name: String,
    temperature: f32,
    maximum_temperature: f32,
    critical_temperature: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Apiv2Schema)]
pub struct System {
    cpu: Vec<Cpu>,
    disk: Vec<Disk>,
    info: OsInfo,
    memory: Memory,
    network: Vec<Network>,
    process: Vec<Process>,
    temperature: Vec<Temperature>,
    unix_time_seconds: u64,
}

pub fn system() -> System {
    System {
        cpu: cpu(),
        disk: disk(),
        info: info(),
        memory: memory(),
        network: network(),
        process: process(),
        temperature: temperature(),
        unix_time_seconds: unix_time_seconds(),
    }
}

#[cached(time = 5)]
pub fn cpu() -> Vec<Cpu> {
    let mut system = SYSTEM.lock().unwrap();
    system.refresh_cpu();

    system
        .cpus()
        .iter()
        .map(|cpu| Cpu {
            name: cpu.name().into(),
            usage: cpu.cpu_usage(),
            frequency: cpu.frequency(),
            vendor_id: cpu.vendor_id().into(),
            brand: cpu.brand().into(),
        })
        .collect::<Vec<Cpu>>()
}

#[cached(time = 5)]
pub fn disk() -> Vec<Disk> {
    let mut system = SYSTEM.lock().unwrap();
    system.refresh_disks_list();
    system.refresh_disks();

    system
        .disks()
        .iter()
        .map(|disk| Disk {
            name: disk.name().to_str().unwrap_or_default().into(),
            filesystem_type: std::str::from_utf8(disk.file_system())
                .unwrap_or_default()
                .into(),
            disk_type: format!("{:?}", disk.type_()),
            mount_point: disk.mount_point().to_str().unwrap_or_default().into(),
            available_space_B: disk.available_space().into(),
            total_space_B: disk.total_space().into(),
        })
        .collect::<Vec<Disk>>()
}

#[cached(time = 5)]
pub fn info() -> OsInfo {
    let system = SYSTEM.lock().unwrap();

    OsInfo {
        system_name: system.name().unwrap_or_default(),
        kernel_version: system.kernel_version().unwrap_or_default(),
        os_version: system.os_version().unwrap_or_default(),
        host_name: system.host_name().unwrap_or_default(),
    }
}

#[cached(time = 5)]
pub fn memory() -> Memory {
    let mut system = SYSTEM.lock().unwrap();
    system.refresh_memory();

    Memory {
        ram: MemoryUsage {
            used_kB: system.used_memory(),
            total_kB: system.total_memory(),
        },
        swap: MemoryUsage {
            used_kB: system.used_swap(),
            total_kB: system.total_swap(),
        },
    }
}

#[cached(time = 5)]
pub fn network() -> Vec<Network> {
    let mut system = SYSTEM.lock().unwrap();
    system.refresh_networks();
    system.refresh_networks_list();

    let pnet_interfaces = pnet::datalink::interfaces();

    system
        .networks()
        .iter()
        .map(|(name, network)| {
            let mut pnet_interface = pnet::datalink::NetworkInterface {
                name: Default::default(),
                description: Default::default(),
                index: Default::default(),
                mac: Default::default(),
                ips: Default::default(),
                flags: Default::default(),
            };

            if let Some(interface) = pnet_interfaces
                .iter()
                .find(|interface| &interface.name == name)
            {
                pnet_interface = interface.clone();
            }

            Network {
                name: name.into(),
                description: pnet_interface.description.clone(),

                mac: pnet_interface
                    .mac
                    .unwrap_or(pnet::datalink::MacAddr::zero())
                    .to_string(),
                ips: pnet_interface.ips.iter().map(|ip| ip.to_string()).collect(),

                is_up: pnet_interface.is_up(),
                is_loopback: pnet_interface.is_loopback(),

                received_B: network.received(),
                total_received_B: network.total_received(),

                transmitted_B: network.transmitted(),
                total_transmitted_B: network.total_transmitted(),

                packets_received: network.packets_received(),
                total_packets_received: network.total_packets_received(),

                packets_transmitted: network.packets_transmitted(),
                total_packets_transmitted: network.total_packets_transmitted(),

                errors_on_received: network.errors_on_received(),
                total_errors_on_received: network.total_errors_on_received(),

                errors_on_transmitted: network.errors_on_transmitted(),
                total_errors_on_transmitted: network.total_errors_on_transmitted(),
            }
        })
        .collect::<Vec<Network>>()
}

#[cached(time = 5)]
pub fn process() -> Vec<Process> {
    let mut system = SYSTEM.lock().unwrap();
    system.refresh_processes();
    system
        .processes()
        .values()
        .map(|process| {
            let disk_usage = process.disk_usage();
            Process {
                name: process.name().into(),
                pid: process.pid().as_u32(),
                status: format!("{:?}", process.status()),
                command: process.cmd().into(),
                executable_path: process.exe().to_str().unwrap_or_default().into(),
                environment: process.environ().into(),
                working_directory: process.cwd().to_str().unwrap_or_default().into(),
                root_directory: process.root().to_str().unwrap_or_default().into(),
                used_memory_kB: process.memory(),
                virtual_memory_kB: process.virtual_memory(),
                parent_process: process.parent().and_then(|pid| Some(pid.as_u32())),
                running_time: process.start_time(),
                cpu_usage: process.cpu_usage(),
                disk_usage: DiskUsage {
                    total_written_bytes: disk_usage.total_written_bytes,
                    written_bytes: disk_usage.written_bytes,
                    total_read_bytes: disk_usage.total_read_bytes,
                    read_bytes: disk_usage.read_bytes,
                },
            }
        })
        .collect::<Vec<Process>>()
}

#[cached(time = 5)]
pub fn temperature() -> Vec<Temperature> {
    let mut system = SYSTEM.lock().unwrap();
    system.refresh_components();
    system.refresh_components_list();

    system
        .components()
        .iter()
        .map(|component| Temperature {
            name: component.label().into(),
            temperature: component.temperature(),
            maximum_temperature: component.max(),
            critical_temperature: component.critical(),
        })
        .collect::<Vec<Temperature>>()
}

pub fn unix_time_seconds() -> u64 {
    return match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(time) => time.as_secs(),
        Err(error) => {
            warn!("SystemTime before UNIX EPOCH: {error}");
            0
        }
    };
}
