use paperclip::actix::Apiv2Schema;
use serde::Serialize;

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
    pub port_name: String,
    /// The long name of the serial port
    pub port_by_path: Option<String>,
    /// The hardware device type that exposes this port
    pub port_type: SerialPortType,
    /// Udev information from the device
    pub udev: Option<serde_json::Value>,
}

impl PortInfo {
    fn from(port: &serialport::SerialPortInfo, include_udev: bool) -> Self {
        let mut udev_enumerator = udev::Enumerator::new().unwrap();
        let udev_result = udev_enumerator.scan_devices().unwrap();
        let udev_device = udev_result
            .filter(|device| device.devnode().is_some())
            .find(|device| device.devnode().unwrap().to_str().unwrap() == port.port_name);

        let by_path = if let Some(udev_device) = &udev_device {
            let udev_entry = udev_device
                .properties()
                .find(|property| property.name() == "DEVLINKS");
            let result = udev_entry
                .unwrap()
                .value()
                .to_str()
                .unwrap()
                .split(' ')
                .find(|link| link.contains("by-path"))
                .unwrap_or_default()
                .to_string();
            Some(result)
        } else {
            None
        };

        PortInfo {
            port_name: port.port_name.clone(),
            port_by_path: by_path,
            port_type: SerialPortType::from(&port.port_type),
            udev: if include_udev && udev_device.is_some() {
                Some(crate::features::udev::generate_serde_from_device(
                    &udev_device.unwrap(),
                ))
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
