use cached::proc_macro::cached;
use paperclip::actix::Apiv2Schema;
use serde::Serialize;

use crate::features::common::UsbPortInfo;

/// Information about a USB device
#[derive(Clone, Debug, Serialize, Apiv2Schema)]
pub struct UsbDevice {
    /// USB device information (VID, PID, manufacturer, etc.)
    #[serde(flatten)]
    pub info: UsbPortInfo,
    /// USB device class
    pub device_class: u8,
    /// USB device subclass
    pub device_subclass: u8,
    /// USB device protocol
    pub device_protocol: u8,
    /// USB specification version (e.g., "2.0", "3.0")
    pub usb_version: String,
    /// Device speed (e.g., "Low", "Full", "High", "Super")
    pub speed: String,
    /// Number of configurations
    pub num_configurations: u8,
}

/// List of USB devices
#[derive(Clone, Debug, Serialize, Apiv2Schema)]
pub struct UsbDevices {
    /// List of USB devices found on the system
    pub devices: Vec<UsbDevice>,
}

/// Get USB speed as a human-readable string
fn speed_to_string(speed: rusb::Speed) -> String {
    match speed {
        rusb::Speed::Low => "Low (1.5 Mbps)".to_string(),
        rusb::Speed::Full => "Full (12 Mbps)".to_string(),
        rusb::Speed::High => "High (480 Mbps)".to_string(),
        rusb::Speed::Super => "Super (5 Gbps)".to_string(),
        rusb::Speed::SuperPlus => "SuperPlus (10 Gbps)".to_string(),
        _ => "Unknown".to_string(),
    }
}

/// Format USB version from rusb::Version
fn format_usb_version(version: rusb::Version) -> String {
    let (major, minor, patch) = (version.major(), version.minor(), version.sub_minor());
    if patch == 0 {
        format!("{}.{}", major, minor)
    } else {
        format!("{}.{}.{}", major, minor, patch)
    }
}

/// List all USB devices on the system
#[cached(time = 5)]
pub fn usb_devices() -> UsbDevices {
    let devices = match rusb::devices() {
        Ok(device_list) => device_list
            .iter()
            .filter_map(|device| {
                let descriptor = device.device_descriptor().ok()?;

                Some(UsbDevice {
                    info: UsbPortInfo::from_rusb(&device),
                    device_class: descriptor.class_code(),
                    device_subclass: descriptor.sub_class_code(),
                    device_protocol: descriptor.protocol_code(),
                    usb_version: format_usb_version(descriptor.usb_version()),
                    speed: speed_to_string(device.speed()),
                    num_configurations: descriptor.num_configurations(),
                })
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    UsbDevices { devices }
}
