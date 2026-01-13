use paperclip::actix::Apiv2Schema;
use serde::Serialize;
use std::path::PathBuf;
use usb_ids::FromId;

/// USB device information shared between USB and Serial features
#[derive(Clone, Debug, Serialize, Apiv2Schema)]
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
    /// USB port path (e.g., "1-1.2", "3-1.4.1") - represents the physical USB topology
    pub port_path: Option<String>,
    /// USB bus number
    pub bus_number: Option<u8>,
    /// USB device address on the bus
    pub device_address: Option<u8>,
}

impl UsbPortInfo {
    /// Create UsbPortInfo from serialport::UsbPortInfo (for serial port compatibility)
    #[allow(dead_code)]
    pub fn from_serialport(usb_port_info: &serialport::UsbPortInfo) -> Self {
        UsbPortInfo {
            vid: usb_port_info.vid,
            pid: usb_port_info.pid,
            serial_number: usb_port_info.serial_number.clone(),
            manufacturer: usb_port_info.manufacturer.clone(),
            product: usb_port_info.product.clone(),
            port_path: None,
            bus_number: None,
            device_address: None,
        }
    }

    /// Create UsbPortInfo from rusb device
    pub fn from_rusb<T: rusb::UsbContext>(device: &rusb::Device<T>) -> Self {
        let descriptor = device.device_descriptor().ok();
        let port_path = Self::get_port_path(device);
        let bus_number = device.bus_number();
        let device_address = device.address();
        let vid = descriptor.as_ref().map(|d| d.vendor_id()).unwrap_or(0);
        let pid = descriptor.as_ref().map(|d| d.product_id()).unwrap_or(0);

        // Try to read from sysfs first (doesn't require permissions)
        let sysfs_info = port_path.as_ref().and_then(Self::read_sysfs_info);

        // Fall back to rusb device handle if sysfs didn't work
        let (mut manufacturer, mut product, serial_number) = if let Some(info) = sysfs_info {
            info
        } else if let Some(ref desc) = descriptor {
            if let Ok(handle) = device.open() {
                let manufacturer = handle
                    .read_manufacturer_string_ascii(desc)
                    .ok()
                    .filter(|s| !s.is_empty());
                let product = handle
                    .read_product_string_ascii(desc)
                    .ok()
                    .filter(|s| !s.is_empty());
                let serial = handle
                    .read_serial_number_string_ascii(desc)
                    .ok()
                    .filter(|s| !s.is_empty());
                (manufacturer, product, serial)
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

        // Fall back to USB ID database for vendor/product names
        if manufacturer.is_none() {
            manufacturer = usb_ids::Vendor::from_id(vid).map(|v| v.name().to_string());
        }
        if product.is_none() {
            product = usb_ids::Device::from_vid_pid(vid, pid).map(|d| d.name().to_string());
        }

        UsbPortInfo {
            vid,
            pid,
            serial_number,
            manufacturer,
            product,
            port_path,
            bus_number: Some(bus_number),
            device_address: Some(device_address),
        }
    }

    /// Get the USB port path (e.g., "1-1.2") from a rusb device
    fn get_port_path<T: rusb::UsbContext>(device: &rusb::Device<T>) -> Option<String> {
        let bus = device.bus_number();
        let port_numbers = device.port_numbers().ok()?;

        if port_numbers.is_empty() {
            // Root hub - just return bus number
            return Some(format!("{}-0", bus));
        }

        let port_chain: Vec<String> = port_numbers.iter().map(|p| p.to_string()).collect();
        Some(format!("{}-{}", bus, port_chain.join(".")))
    }

    /// Read USB device info from sysfs (doesn't require root permissions)
    fn read_sysfs_info(
        port_path: &String,
    ) -> Option<(Option<String>, Option<String>, Option<String>)> {
        let sysfs_path = PathBuf::from("/sys/bus/usb/devices").join(port_path);

        if !sysfs_path.exists() {
            return None;
        }

        let manufacturer = std::fs::read_to_string(sysfs_path.join("manufacturer"))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let product = std::fs::read_to_string(sysfs_path.join("product"))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let serial = std::fs::read_to_string(sysfs_path.join("serial"))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        // Only return Some if we got at least one piece of info
        if manufacturer.is_some() || product.is_some() || serial.is_some() {
            Some((manufacturer, product, serial))
        } else {
            None
        }
    }
}
