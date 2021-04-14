use std::sync::{Arc, Mutex};

use pnet;
use serde::{
    ser::{SerializeMap, SerializeSeq, SerializeStruct},
    Serialize, Serializer,
};
use sysinfo::{
    ComponentExt, DiskExt, NetworkExt, NetworksExt, ProcessExt, ProcessorExt, System, SystemExt,
};

lazy_static! {
    static ref SYSTEM: Arc<Mutex<System>> = Arc::new(Mutex::new(System::new()));
}

#[derive(Debug)]
pub enum SystemType {
    Cpu,
    Disk,
    Everything,
    Info,
    Memory,
    Network,
    Process,
    Temperature,
}

pub fn generate_serde_value(system_type: SystemType) -> serde_json::Value {
    match system_type {
        SystemType::Everything => {
            let memory = serde_json::json!({
                "cpu": generate_serde_value(SystemType::Cpu),
                "disk": generate_serde_value(SystemType::Disk),
                "info": generate_serde_value(SystemType::Info),
                "memory": generate_serde_value(SystemType::Memory),
                "network": generate_serde_value(SystemType::Network),
                "process": generate_serde_value(SystemType::Process),
                "temperature": generate_serde_value(SystemType::Temperature),
            });

            return serde_json::to_value(&memory).unwrap();
        }

        SystemType::Cpu => {
            let mut system = SYSTEM.lock().unwrap();
            system.refresh_cpu();

            let cpus = system
                .get_processors()
                .iter()
                .map(|cpu| {
                    serde_json::json!(
                        {
                            "name": cpu.get_name(),
                            "usage": cpu.get_cpu_usage(),
                            "frequency": cpu.get_frequency(),
                            "vendor_id": cpu.get_vendor_id(),
                            "brand": cpu.get_brand(),
                        }
                    )
                })
                .collect::<Vec<serde_json::Value>>();

            return serde_json::to_value(&cpus).unwrap();
        }

        SystemType::Disk => {
            let mut system = SYSTEM.lock().unwrap();
            system.refresh_disks_list();
            system.refresh_disks();

            let disks = system
                .get_disks()
                .iter()
                .map(|disk| {
                    serde_json::json!(
                    {
                        "name": disk.get_name().to_str(),
                        "filesystem_type": std::str::from_utf8(disk.get_file_system()).unwrap_or_default(),
                        "type": format!("{:?}", disk.get_type()),
                        "mount_point": &disk.get_mount_point(),
                        "available_space_B": &disk.get_available_space(),
                        "total_space_B": &disk.get_total_space(),
                    })
                })
                .collect::<Vec<serde_json::Value>>();

            return serde_json::to_value(&disks).unwrap();
        }

        SystemType::Info => {
            let system = SYSTEM.lock().unwrap();

            let info = serde_json::json!(
            {
                "system_name": system.get_name(),
                "kernel_version": system.get_kernel_version(),
                "os_version": system.get_os_version(),
                "host_name": system.get_host_name(),
            });

            return serde_json::to_value(&info).unwrap();
        }

        SystemType::Memory => {
            let mut system = SYSTEM.lock().unwrap();
            system.refresh_memory();

            let memory = serde_json::json!(
            {
                "ram": {
                    "used_kB" : system.get_used_memory(),
                    "total_kB" : system.get_total_memory(),
                },
                "swap": {
                    "used_kB" : system.get_used_swap(),
                    "total_kB" : system.get_total_swap(),
                },
            });

            return serde_json::to_value(&memory).unwrap();
        }

        SystemType::Network => {
            let mut system = SYSTEM.lock().unwrap();
            system.refresh_networks();
            system.refresh_networks_list();

            let pnet_interfaces = pnet::datalink::interfaces();

            let networks = system
                .get_networks()
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

                    serde_json::json!({
                        "name": name,
                        "description": pnet_interface.description,

                        "mac": pnet_interface.mac.unwrap_or(pnet::datalink::MacAddr::zero()).to_string(),
                        "ips": pnet_interface.ips,

                        "is_up": pnet_interface.is_up(),
                        "is_loopback": pnet_interface.is_loopback(),

                        "received_B": network.get_received(),
                        "total_received_B": network.get_total_received(),

                        "transmitted_B": network.get_transmitted(),
                        "total_transmitted_B": network.get_total_transmitted(),

                        "packets_received": network.get_packets_received(),
                        "total_packets_received": network.get_total_packets_received(),

                        "packets_transmitted": network.get_packets_transmitted(),
                        "total_packets_transmitted": network.get_total_packets_transmitted(),

                        "errors_on_received": network.get_errors_on_received(),
                        "total_errors_on_received": network.get_total_errors_on_received(),

                        "errors_on_transmitted": network.get_errors_on_transmitted(),
                        "total_errors_on_transmitted": network.get_total_errors_on_transmitted(),
                    })
                })
                .collect::<Vec<serde_json::Value>>();

            return serde_json::to_value(&networks).unwrap();
        }

        SystemType::Process => {
            let mut system = SYSTEM.lock().unwrap();
            system.refresh_processes();
            let processes = system
                .get_processes()
                .values()
                .map(|process| {
                    let disk_usage = process.disk_usage();
                    serde_json::json!({
                        "name": process.name(),
                        "pid": process.pid(),
                        "status": format!("{:?}", process.status()),
                        "command": process.cmd(),
                        "executable_path": process.exe(),
                        "environment": process.environ(),
                        "working_directory": process.cwd(),
                        "root_directory": process.root(),
                        "used_memory_kB": process.memory(),
                        "virtual_memory_kB": process.virtual_memory(),
                        "parent_process": process.parent(),
                        "running_time": process.start_time(),
                        "cpu_usage": process.cpu_usage(),
                        "disk_usage": serde_json::json!({
                            "total_written_bytes": disk_usage.total_written_bytes,
                            "written_bytes": disk_usage.written_bytes,
                            "total_read_bytes": disk_usage.total_read_bytes,
                            "read_bytes": disk_usage.read_bytes,
                        }),
                    })
                })
                .collect::<Vec<serde_json::Value>>();

            return serde_json::to_value(&processes).unwrap();
        }

        SystemType::Temperature => {
            let mut system = SYSTEM.lock().unwrap();
            system.refresh_components();
            system.refresh_components_list();

            let temperatures = system
                .get_components()
                .iter()
                .map(|component| {
                    serde_json::json!(
                    {
                        "name" : component.get_label(),
                        "temperature" : component.get_temperature(),
                        "maximum_temperature" : component.get_max(),
                        "critical_temperature" : component.get_critical(),
                    })
                })
                .collect::<Vec<serde_json::Value>>();

            return serde_json::to_value(&temperatures).unwrap();
        }
    }
}
