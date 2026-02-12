use std::{sync::OnceLock, time::Duration};

use paperclip::actix::Apiv2Schema;
use serde::Serialize;
use sysinfo::{
    Components, CpuRefreshKind, Disks, MemoryRefreshKind, Networks, ProcessRefreshKind,
    ProcessesToUpdate, RefreshKind, System as SysSystem, UpdateKind, MINIMUM_CPU_UPDATE_INTERVAL,
};
use tokio::sync::watch;
use tracing::*;

static SYSTEM: OnceLock<SystemInner> = OnceLock::new();

#[derive(Debug)]
struct SystemInner {
    cpu: watch::Receiver<Vec<Cpu>>,
    disk: watch::Receiver<Vec<Disk>>,
    info: watch::Receiver<OsInfo>,
    memory: watch::Receiver<Memory>,
    network: watch::Receiver<Vec<Network>>,
    process: watch::Receiver<Vec<Process>>,
    temperature: watch::Receiver<Vec<Temperature>>,
}

struct Sampler {
    system: SysSystem,
    components: Components,
    disks: Disks,
    networks: Networks,
    cpu_tx: watch::Sender<Vec<Cpu>>,
    disk_tx: watch::Sender<Vec<Disk>>,
    info_tx: watch::Sender<OsInfo>,
    memory_tx: watch::Sender<Memory>,
    network_tx: watch::Sender<Vec<Network>>,
    process_tx: watch::Sender<Vec<Process>>,
    temperature_tx: watch::Sender<Vec<Temperature>>,
}

pub fn start(sample_interval: Duration) {
    let sample_interval = std::cmp::max(sample_interval, MINIMUM_CPU_UPDATE_INTERVAL);

    let (mut context, channels) = Sampler::new();

    SYSTEM
        .set(channels)
        .expect("System actor already initialized");

    tokio::spawn(async move {
        // Wait for CPU delta baseline before first sample
        tokio::time::sleep(MINIMUM_CPU_UPDATE_INTERVAL).await;
        context.sample_and_send();

        let mut interval = tokio::time::interval(sample_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        interval.tick().await; // Consume immediate first tick

        loop {
            interval.tick().await;
            context.sample_and_send();
            trace!("System actor completed sample cycle");
        }
    });

    info!(
        "System actor started with {}ms sample interval",
        sample_interval.as_millis()
    );
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

#[derive(Clone, Debug, Default, Serialize, Apiv2Schema)]
pub struct OsInfo {
    system_name: String,
    kernel_version: String,
    os_version: String,
    host_name: String,
}

#[derive(Clone, Debug, Default, Serialize, Apiv2Schema)]
pub struct MemoryUsage {
    used_kB: u64,
    total_kB: u64,
}

#[derive(Clone, Debug, Default, Serialize, Apiv2Schema)]
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

impl Sampler {
    fn new() -> (Self, SystemInner) {
        let (cpu_tx, cpu_rx) = watch::channel(Vec::new());
        let (disk_tx, disk_rx) = watch::channel(Vec::new());
        let (memory_tx, memory_rx) = watch::channel(Memory::default());
        let (network_tx, network_rx) = watch::channel(Vec::new());
        let (process_tx, process_rx) = watch::channel(Vec::new());
        let (temperature_tx, temperature_rx) = watch::channel(Vec::new());

        let mut system = SysSystem::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );

        // Establish CPU baselines for delta calculations
        system.refresh_cpu_usage();
        system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_cpu().without_tasks(),
        );

        let components = Components::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();

        let initial_info = OsInfo {
            system_name: SysSystem::name().unwrap_or_default(),
            kernel_version: SysSystem::kernel_version().unwrap_or_default(),
            os_version: SysSystem::os_version().unwrap_or_default(),
            host_name: SysSystem::host_name().unwrap_or_default(),
        };
        let (info_tx, info_rx) = watch::channel(initial_info);

        let context = Self {
            system,
            components,
            disks,
            networks,
            cpu_tx,
            disk_tx,
            info_tx,
            memory_tx,
            network_tx,
            process_tx,
            temperature_tx,
        };

        let channels = SystemInner {
            cpu: cpu_rx,
            disk: disk_rx,
            info: info_rx,
            memory: memory_rx,
            network: network_rx,
            process: process_rx,
            temperature: temperature_rx,
        };

        (context, channels)
    }

    fn sample_and_send(&mut self) {
        self.system.refresh_memory();
        self.system.refresh_cpu_all();
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_memory()
                .with_cpu()
                .with_disk_usage()
                .with_exe(UpdateKind::OnlyIfNotSet)
                .with_cmd(UpdateKind::OnlyIfNotSet)
                .with_environ(UpdateKind::OnlyIfNotSet)
                .with_cwd(UpdateKind::OnlyIfNotSet)
                .with_root(UpdateKind::OnlyIfNotSet)
                .with_user(UpdateKind::OnlyIfNotSet)
                .without_tasks(),
        );

        self.disks.refresh(true);
        self.networks.refresh(true);
        self.components.refresh(true);

        let _ = self.cpu_tx.send(self.cpu());
        let _ = self.disk_tx.send(self.disk());
        let _ = self.info_tx.send(self.info());
        let _ = self.memory_tx.send(self.memory());
        let _ = self.network_tx.send(self.network());
        let _ = self.process_tx.send(self.process());
        let _ = self.temperature_tx.send(self.temperature());
    }

    fn cpu(&self) -> Vec<Cpu> {
        self.system
            .cpus()
            .iter()
            .map(|cpu| Cpu {
                name: cpu.name().into(),
                usage: cpu.cpu_usage(),
                frequency: cpu.frequency(),
                vendor_id: cpu.vendor_id().into(),
                brand: cpu.brand().into(),
            })
            .collect()
    }

    fn disk(&self) -> Vec<Disk> {
        self.disks
            .iter()
            .map(|disk| Disk {
                name: disk.name().to_str().unwrap_or_default().into(),
                filesystem_type: disk.file_system().to_str().unwrap_or_default().into(),
                disk_type: format!("{:?}", disk.kind()),
                mount_point: disk.mount_point().to_str().unwrap_or_default().into(),
                available_space_B: disk.available_space(),
                total_space_B: disk.total_space(),
            })
            .collect()
    }

    fn info(&self) -> OsInfo {
        OsInfo {
            system_name: SysSystem::name().unwrap_or_default(),
            kernel_version: SysSystem::kernel_version().unwrap_or_default(),
            os_version: SysSystem::os_version().unwrap_or_default(),
            host_name: SysSystem::host_name().unwrap_or_default(),
        }
    }

    fn memory(&self) -> Memory {
        Memory {
            ram: MemoryUsage {
                used_kB: self.system.used_memory().div_ceil(1024),
                total_kB: self.system.total_memory().div_ceil(1024),
            },
            swap: MemoryUsage {
                used_kB: self.system.used_swap().div_ceil(1024),
                total_kB: self.system.total_swap().div_ceil(1024),
            },
        }
    }

    fn network(&self) -> Vec<Network> {
        let pnet_interfaces = pnet::datalink::interfaces();

        self.networks
            .iter()
            .map(|(name, network)| {
                let pnet_interface = pnet_interfaces
                    .iter()
                    .find(|interface| &interface.name == name)
                    .cloned()
                    .unwrap_or_else(|| pnet::datalink::NetworkInterface {
                        name: Default::default(),
                        description: Default::default(),
                        index: Default::default(),
                        mac: Default::default(),
                        ips: Default::default(),
                        flags: Default::default(),
                    });

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
            .collect()
    }

    fn process(&self) -> Vec<Process> {
        self.system
            .processes()
            .values()
            .map(|process| {
                let disk_usage = process.disk_usage();
                Process {
                    name: process.name().to_string_lossy().into_owned(),
                    pid: process.pid().as_u32(),
                    status: format!("{:?}", process.status()),
                    command: process
                        .cmd()
                        .iter()
                        .map(|s| s.to_string_lossy().into_owned())
                        .collect(),
                    executable_path: process
                        .exe()
                        .and_then(|p| p.to_str())
                        .unwrap_or_default()
                        .into(),
                    environment: process
                        .environ()
                        .iter()
                        .map(|s| s.to_string_lossy().into_owned())
                        .collect(),
                    working_directory: process
                        .cwd()
                        .and_then(|p| p.to_str())
                        .unwrap_or_default()
                        .into(),
                    root_directory: process
                        .root()
                        .and_then(|p| p.to_str())
                        .unwrap_or_default()
                        .into(),
                    used_memory_kB: process.memory().div_ceil(1024),
                    virtual_memory_kB: process.virtual_memory().div_ceil(1024),
                    parent_process: process.parent().map(|pid| pid.as_u32()),
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
            .collect()
    }

    fn temperature(&self) -> Vec<Temperature> {
        self.components
            .iter()
            .map(|component| Temperature {
                name: component.label().into(),
                temperature: component.temperature().unwrap_or(0.0),
                maximum_temperature: component.max().unwrap_or(0.0),
                critical_temperature: component.critical(),
            })
            .collect()
    }
}

fn inner() -> &'static SystemInner {
    SYSTEM
        .get()
        .expect("System actor not initialized. Call system::start() in main() first.")
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

pub fn cpu() -> Vec<Cpu> {
    inner().cpu.borrow().clone()
}

pub fn disk() -> Vec<Disk> {
    inner().disk.borrow().clone()
}

pub fn info() -> OsInfo {
    inner().info.borrow().clone()
}

pub fn memory() -> Memory {
    inner().memory.borrow().clone()
}

pub fn network() -> Vec<Network> {
    inner().network.borrow().clone()
}

pub fn process() -> Vec<Process> {
    inner().process.borrow().clone()
}

pub fn temperature() -> Vec<Temperature> {
    inner().temperature.borrow().clone()
}

pub fn unix_time_seconds() -> u64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(time) => time.as_secs(),
        Err(error) => {
            warn!("SystemTime before UNIX EPOCH: {error}");
            0
        }
    }
}
