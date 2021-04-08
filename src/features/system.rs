use std::sync::{Arc, Mutex};

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

            let processes = system
                .get_processors()
                .iter()
                .map(|cpu| GenericSerializer::from(cpu))
                .collect::<Vec<GenericSerializer<sysinfo::Processor>>>();

            return serde_json::to_value(&processes).unwrap();
        }

        SystemType::Disk => {
            let mut system = SYSTEM.lock().unwrap();
            system.refresh_disks_list();
            system.refresh_disks();

            let disks = system
                .get_disks()
                .iter()
                .map(|disk| GenericSerializer::from(disk))
                .collect::<Vec<GenericSerializer<sysinfo::Disk>>>();

            return serde_json::to_value(&disks).unwrap();
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

            let networks = system
                .get_networks()
                .iter()
                .map(|(name, network)| {
                    serde_json::to_value(&GenericSerializer::from(&(name, network))).unwrap()
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
                .map(|process| GenericSerializer::from(process))
                .collect::<Vec<GenericSerializer<sysinfo::Process>>>();

            return serde_json::to_value(&processes).unwrap();
        }

        SystemType::Temperature => {
            let mut system = SYSTEM.lock().unwrap();
            system.refresh_components();
            system.refresh_components_list();

            let temperatures = system
                .get_components()
                .iter()
                .map(|component| GenericSerializer::from(component))
                .collect::<Vec<GenericSerializer<sysinfo::Component>>>();

            return serde_json::to_value(&temperatures).unwrap();
        }
    }
}

struct GenericSerializer<'a, T: 'a>(&'a T);

impl<'a, T> GenericSerializer<'a, T>
where
    GenericSerializer<'a, T>: Serialize,
{
    #[inline(always)]
    pub fn from(value: &'a T) -> Self {
        GenericSerializer(value)
    }
}

impl<'a> Serialize for GenericSerializer<'a, sysinfo::Process> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Process", 14)?;
        state.serialize_field("name", &self.0.name())?;
        state.serialize_field("pid", &self.0.pid())?;
        state.serialize_field("status", &GenericSerializer::from(&self.0.status()))?;
        state.serialize_field("command", &self.0.cmd())?;
        state.serialize_field("executable_path", &self.0.exe())?;
        state.serialize_field("environment", &self.0.environ())?;
        state.serialize_field("working_directory", &self.0.cwd())?;
        state.serialize_field("root_directory", &self.0.root())?;
        state.serialize_field("used_memory_kB", &self.0.memory())?;
        state.serialize_field("virtual_memory_kB", &self.0.virtual_memory())?;
        state.serialize_field("parent_process", &self.0.parent())?;
        state.serialize_field("running_time", &self.0.start_time())?;
        state.serialize_field("cpu_usage", &self.0.cpu_usage())?;
        state.serialize_field("disk_usage", &GenericSerializer::from(&self.0.disk_usage()))?;
        state.end()
    }
}

impl<'a> Serialize for GenericSerializer<'a, sysinfo::ProcessStatus> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            sysinfo::ProcessStatus::Idle => {
                serializer.serialize_unit_variant("ProcessStatus", 0, "Idle")
            }

            sysinfo::ProcessStatus::Run => {
                serializer.serialize_unit_variant("ProcessStatus", 1, "Run")
            }

            sysinfo::ProcessStatus::Sleep => {
                serializer.serialize_unit_variant("ProcessStatus", 2, "Sleep")
            }

            sysinfo::ProcessStatus::Stop => {
                serializer.serialize_unit_variant("ProcessStatus", 3, "Stop")
            }

            sysinfo::ProcessStatus::Zombie => {
                serializer.serialize_unit_variant("ProcessStatus", 4, "Zombie")
            }

            sysinfo::ProcessStatus::Tracing => {
                serializer.serialize_unit_variant("ProcessStatus", 5, "Tracing")
            }

            sysinfo::ProcessStatus::Dead => {
                serializer.serialize_unit_variant("ProcessStatus", 6, "Dead")
            }

            sysinfo::ProcessStatus::Wakekill => {
                serializer.serialize_unit_variant("ProcessStatus", 7, "Wakekill")
            }

            sysinfo::ProcessStatus::Waking => {
                serializer.serialize_unit_variant("ProcessStatus", 8, "Waking")
            }

            sysinfo::ProcessStatus::Parked => {
                serializer.serialize_unit_variant("ProcessStatus", 9, "Parked")
            }

            sysinfo::ProcessStatus::Unknown(_) => {
                serializer.serialize_unit_variant("ProcessStatus", 10, "Unknown")
            }
        }
    }
}

impl<'a> Serialize for GenericSerializer<'a, sysinfo::DiskUsage> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("DiskUsage", 4)?;
        state.serialize_field("total_written_bytes", &self.0.total_written_bytes)?;
        state.serialize_field("written_bytes", &self.0.written_bytes)?;
        state.serialize_field("total_read_bytes", &self.0.total_read_bytes)?;
        state.serialize_field("read_bytes", &self.0.read_bytes)?;
        state.end()
    }
}

impl<'a> Serialize for GenericSerializer<'a, sysinfo::Component> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Temperature", 4)?;
        state.serialize_field("name", &self.0.get_label())?;
        state.serialize_field("temperature", &self.0.get_temperature())?;
        state.serialize_field("maximum_temperature", &self.0.get_max())?;
        state.serialize_field("critical_temperature", &self.0.get_critical())?;
        state.end()
    }
}

impl<'a> Serialize for GenericSerializer<'a, sysinfo::Processor> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Cpu", 5)?;
        state.serialize_field("name", &self.0.get_name())?;
        state.serialize_field("usage", &self.0.get_cpu_usage())?;
        state.serialize_field("frequency", &self.0.get_frequency())?;
        state.serialize_field("vendor_id", &self.0.get_vendor_id())?;
        state.serialize_field("brand", &self.0.get_brand())?;
        state.end()
    }
}

impl<'a> Serialize for GenericSerializer<'a, sysinfo::Disk> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Disk", 6)?;
        state.serialize_field("name", &self.0.get_name().to_str())?;
        state.serialize_field(
            "filesystem_type",
            &std::str::from_utf8(self.0.get_file_system()).unwrap_or_default(),
        )?;
        state.serialize_field("type", &GenericSerializer::from(&self.0.get_type()))?;
        state.serialize_field("mount_point", &self.0.get_mount_point())?;
        state.serialize_field("available_space_B", &self.0.get_available_space())?;
        state.serialize_field("total_space_B", &self.0.get_total_space())?;
        state.end()
    }
}

impl<'a> Serialize for GenericSerializer<'a, sysinfo::DiskType> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            sysinfo::DiskType::HDD => serializer.serialize_unit_variant("DiskType", 0, "HDD"),

            sysinfo::DiskType::SSD => serializer.serialize_unit_variant("DiskType", 1, "SSD"),

            sysinfo::DiskType::Removable => {
                serializer.serialize_unit_variant("DiskType", 2, "Removable")
            }

            sysinfo::DiskType::Unknown(_) => {
                serializer.serialize_unit_variant("DiskType", 10, "Unknown")
            }
        }
    }
}

impl<'a> Serialize for GenericSerializer<'a, (&String, &sysinfo::NetworkData)> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Disk", 13)?;
        let name = &self.0 .0;
        let network = &self.0 .1;

        state.serialize_field("name", &name)?;

        state.serialize_field("received_B", &network.get_received())?;
        state.serialize_field("total_received_B", &network.get_total_received())?;

        state.serialize_field("transmitted_B", &network.get_transmitted())?;
        state.serialize_field("total_transmitted_B", &network.get_total_transmitted())?;

        state.serialize_field("packets_received", &network.get_packets_received())?;
        state.serialize_field(
            "total_packets_received",
            &network.get_total_packets_received(),
        )?;

        state.serialize_field("packets_transmitted", &network.get_packets_transmitted())?;
        state.serialize_field(
            "total_packets_transmitted",
            &network.get_total_packets_transmitted(),
        )?;

        state.serialize_field("errors_on_received", &network.get_errors_on_received())?;
        state.serialize_field(
            "total_errors_on_received",
            &network.get_total_errors_on_received(),
        )?;

        state.serialize_field(
            "errors_on_transmitted",
            &network.get_errors_on_transmitted(),
        )?;
        state.serialize_field(
            "total_errors_on_transmitted",
            &network.get_total_errors_on_transmitted(),
        )?;

        state.end()
    }
}
