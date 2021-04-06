use std::borrow::Borrow;

use serde::{
    ser::{SerializeMap, SerializeStruct},
    Serialize, Serializer,
};
use udev;

struct DeviceUdev<'a> {
    device: &'a udev::Device,
}

impl<'a> From<&'a udev::Device> for DeviceUdev<'a> {
    fn from(device: &'a udev::Device) -> Self {
        Self { device }
    }
}

struct DeviceUdevProperties<'a> {
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

impl<'a> Serialize for DeviceUdev<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("DeviceUdev", 12)?;
        state.serialize_field("initialized", &self.device.is_initialized())?;
        state.serialize_field("device_major_minor_number", &self.device.devnum())?;
        state.serialize_field("system_path", &self.device.syspath())?;
        state.serialize_field("device_path", &self.device.devpath().to_str())?;
        state.serialize_field(
            "device_node",
            &self.device.devnode().and_then(|value| value.to_str()),
        )?;
        state.serialize_field(
            "subsystem_name",
            &self.device.subsystem().and_then(|value| value.to_str()),
        )?;
        state.serialize_field("system_name", &self.device.sysname().to_str())?;
        state.serialize_field("instance_number", &self.device.sysnum())?;
        state.serialize_field(
            "device_type",
            &self.device.devtype().and_then(|value| value.to_str()),
        )?;
        state.serialize_field(
            "driver",
            &self.device.driver().and_then(|value| value.to_str()),
        )?;
        state.serialize_field(
            "action",
            &self.device.action().and_then(|value| value.to_str()),
        )?;
        if let Some(parent) = &self.device.parent() {
            state.serialize_field("parent", &Some(DeviceUdev::from(parent)))?;
        } else {
            let x: Option<&str> = None;
            state.serialize_field("parent", &x)?;
        }
        state.serialize_field("properties", &DeviceUdevProperties::from(self.device))?;
        state.serialize_field("attributes", &DeviceUdevAttributes::from(self.device))?;
        state.end()
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

pub fn generate_serde_value() -> Vec<serde_json::Value> {
    let mut enumerator = udev::Enumerator::new().unwrap();
    let result = enumerator.scan_devices().unwrap();
    let vector: Vec<serde_json::Value> = result
        .map(|device| serde_json::to_value(&DeviceUdev::from(&device)).unwrap())
        .collect();

    return vector;
}
