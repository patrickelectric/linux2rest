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
pub struct PortInfo {
    /// The short name of the serial port
    pub name: String,
    /// The long name of the serial port
    pub by_path: Option<String>,
    /// Time when by_path was created in ms ago
    pub by_path_created_ms_ago: Option<u128>,
    /// Udev properties from the device
    pub udev_properties: Option<serde_json::Value>,
}

impl PortInfo {
    fn fetch_udev(port: &serialport::SerialPortInfo) -> Option<serde_json::Value> {
        match udev::Device::from_syspath(std::path::Path::new(&port.port_name)) {
            Ok(device) => Some(serde_json::json!(crate::features::udev::DeviceUdevProperties::from(&device))),
            Err(error) => {
                warn!("Failed to grab udev information from device: {error:#?}");
                return None
            }
        }
    }

    fn fetch_by_path(device_path: &String) -> (Option<String>, Option<u128>) {
        let mut sym_path = None;
        let mut time_ago_ms = None;
        let by_path_dir = "/dev/serial/by-path";
        let dir = std::fs::read_dir(by_path_dir);
        if dir.is_err() {
            let error = dir.err().unwrap();
            warn!("Failed to look over {by_path_dir}: {error:#?}");
            return (None, None);
        }

        for entry in dir.unwrap() {
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

            if let Ok(time_info) = metadata.and_then(|metadata| metadata.modified()) {
                time_ago_ms = Some(
                    std::time::SystemTime::now()
                        .duration_since(time_info)
                        .unwrap()
                        .as_millis(),
                );
            }
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
            udev_properties: PortInfo::fetch_udev(&port),
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
