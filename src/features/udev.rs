use serde::{
    ser::{SerializeMap, SerializeStruct},
    Serialize, Serializer,
};
use udev;

// Maybe make it global for all features ?
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

pub struct DeviceUdevProperties<'a> {
    device: &'a udev::Device,
}

impl<'a> From<&'a udev::Device> for DeviceUdevProperties<'a> {
    fn from(device: &'a udev::Device) -> Self {
        Self { device }
    }
}

struct DeviceUdevAttributes<'a> {
    device: &'a udev::Device,
}

impl<'a> From<&'a udev::Device> for DeviceUdevAttributes<'a> {
    fn from(device: &'a udev::Device) -> Self {
        Self { device }
    }
}

impl Serialize for DeviceUdevProperties<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = std::collections::HashMap::new();
        for property in self.device.properties() {
            let name = property.name();
            let value = property.value();
            map.insert(
                name.to_str().unwrap().to_string(),
                value.to_str().unwrap().to_string(),
            );
        }
        let mut state = serializer.serialize_map(Some(map.len()))?;
        for (k, v) in map {
            state.serialize_entry(&k, &v)?;
        }
        state.end()
    }
}

impl Serialize for DeviceUdevAttributes<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = std::collections::HashMap::new();
        for attribute in self.device.attributes() {
            let name = attribute.name();
            let value = attribute.value();
            map.insert(
                name.to_str().unwrap().to_string(),
                value.to_str().unwrap().to_string(),
            );
        }
        let mut state = serializer.serialize_map(Some(map.len()))?;
        for (k, v) in map {
            state.serialize_entry(&k, &v)?;
        }
        state.end()
    }
}

/*
#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Device {
    initialized: bool,
    device_major_minor_number: String,
    system_path: String,
    device_path: String,
    device_node: String,
    subsystem_name: String,
    system_name: String,
    instance_number: String,
    device_type: String,
    driver: String,
    action: String,
    parent: Box<Device>,
    properties: std::iter::Map<String, String>,
    attributes: std::iter::Map<String, String>,
}*/

pub fn generate_serde_value() -> Vec<serde_json::Value> {
    let mut enumerator = udev::Enumerator::new().unwrap();
    let result = enumerator.scan_devices().unwrap();
    let vector: Vec<serde_json::Value> = result
        .map(|device| generate_serde_from_device(&device))
        .collect();

    return vector;
}

pub fn generate_serde_from_device(device: &udev::Device) -> serde_json::Value {
    serde_json::json!({
        "initialized": device.is_initialized(),
        "device_major_minor_number": device.devnum(),
        "system_path": device.syspath(),
        "device_path": device.devpath().to_str(),
        "device_node": device.devnode().and_then(|value| value.to_str()),
        "subsystem_name": device.subsystem().and_then(|value| value.to_str()),
        "system_name": device.sysname().to_str(),
        "instance_number": device.sysnum(),
        "device_type": device.devtype().and_then(|value| value.to_str()),
        "driver": device.driver().and_then(|value| value.to_str()),
        "action": device.action().and_then(|value| value.to_str()),
        "parent": match device.parent() {
            None => serde_json::json!(null),
            Some(parent) => generate_serde_from_device(&parent)
        },
        "properties": &DeviceUdevProperties::from(device),
        "attributes": &DeviceUdevAttributes::from(device),
    })
}
