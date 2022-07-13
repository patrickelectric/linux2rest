use paperclip::actix::Apiv2Schema;
use serde::Serialize;

use log::*;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct UsbPortInfo {
    /// Vendor ID
    pub vid: u16,
    /// Product ID
    pub pid: u16,
    /// Serial number (arbitrary string)
    pub serial_number: Option<String>,
    /// Manufacturer (arbitrary string)
    pub manufacturer: Option<String>,
    /// Product name (arbitrary string)
    pub product: Option<String>,
}

impl UsbPortInfo {
    fn from(usb_port_info: &serialport::UsbPortInfo) -> Self {
        UsbPortInfo {
            vid: usb_port_info.vid,
            pid: usb_port_info.pid,
            serial_number: usb_port_info.serial_number.clone(),
            manufacturer: usb_port_info.manufacturer.clone(),
            product: usb_port_info.product.clone(),
        }
    }
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub enum SerialPortType {
    /// The serial port is connected via USB
    UsbPort(UsbPortInfo),
    /// The serial port is connected via PCI (permanent port)
    PciPort,
    /// The serial port is connected via Bluetooth
    BluetoothPort,
    /// It can't be determined how the serial port is connected
    Unknown,
}

impl SerialPortType {
    fn from(port_type: &serialport::SerialPortType) -> Self {
        match port_type {
            serialport::SerialPortType::UsbPort(usb_port_info) => {
                SerialPortType::UsbPort(UsbPortInfo::from(usb_port_info))
            }
            serialport::SerialPortType::PciPort => SerialPortType::PciPort,
            serialport::SerialPortType::BluetoothPort => SerialPortType::BluetoothPort,
            serialport::SerialPortType::Unknown => SerialPortType::Unknown,
        }
    }
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct PortInfo {
    /// The short name of the serial port
    pub name: String,
    /// The long name of the serial port
    pub by_path: Option<String>,
    /// Time when by_path was created in ms ago
    pub by_path_created_ms_ago: Option<u128>,
    /// The hardware device type that exposes this port
    #[serde(rename = "type")]
    pub port_type: SerialPortType,
    /// Udev information from the device
    pub udev: Option<serde_json::Value>,
}

impl PortInfo {
    fn fetch_udev(port: &serialport::SerialPortInfo) -> Option<serde_json::Value> {
        let mut udev_enumerator = udev::Enumerator::new().unwrap();
        let udev_result = udev_enumerator.scan_devices().unwrap();
        udev_result
            .filter(|device| device.devnode().is_some())
            .find(|device| device.devnode().unwrap().to_str().unwrap() == port.port_name)
            .and_then(|device| Some(crate::features::udev::generate_serde_from_device(&device)))
    }

    fn fetch_by_path(device_path: &String) -> (Option<String>, Option<u128>) {
        let mut sym_path = None;
        let mut time_ago_ms = None;
        for entry in std::fs::read_dir("/dev/serial/by-path").unwrap() {
            sym_path = None;

            if let Err(error) = entry {
                warn!("Failed to open serial by-path folder: {error:#?}");
                continue;
            }

            let entry = entry.unwrap();
            let path = entry.path();

            match std::fs::canonicalize(&path) {
                Ok(real_path) => {
                    if real_path.to_string_lossy().to_string() != *device_path {
                        continue;
                    }
                }
                Err(error) => {
                    warn!("Failed to get canonical path for {device_path}: {error:#?}");
                    continue;
                }
            }

            sym_path = Some(path.clone());

            let metadata = std::fs::metadata(&path);
            if let Err(error) = metadata {
                warn!("Failed to get metadata for {path:#?}: {error:#?}");
                break;
            }

            time_ago_ms = Some(
                std::time::SystemTime::now()
                    .duration_since(metadata.unwrap().created().unwrap())
                    .unwrap()
                    .as_millis(),
            );
            break;
        }

        (
            sym_path.and_then(|path| Some(path.to_string_lossy().to_string())),
            time_ago_ms,
        )
    }

    fn from(port: &serialport::SerialPortInfo, include_udev: bool) -> Self {
        let (sym_path, time_ago_ms) = PortInfo::fetch_by_path(&port.port_name.clone());

        PortInfo {
            name: port.port_name.clone(),
            by_path: sym_path,
            by_path_created_ms_ago: time_ago_ms,
            port_type: SerialPortType::from(&port.port_type),
            udev: if include_udev {
                PortInfo::fetch_udev(port)
            } else {
                None
            },
        }
    }
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct SerialPorts {
    ports: Vec<PortInfo>,
}

//device_node
pub fn serial(udev: Option<bool>) -> SerialPorts {
    SerialPorts {
        ports: serialport::available_ports()
            .unwrap_or_default()
            .iter()
            .map(|port| PortInfo::from(port, udev.unwrap_or(false)))
            .collect(),
    }
}
