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

impl<'a> Serialize for GenericSerializer<'a, udev::Device> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("DeviceUdev", 12)?;
        state.serialize_field("initialized", &self.0.is_initialized())?;
        state.serialize_field("device_major_minor_number", &self.0.devnum())?;
        state.serialize_field("system_path", &self.0.syspath())?;
        state.serialize_field("device_path", &self.0.devpath().to_str())?;
        state.serialize_field(
            "device_node",
            &self.0.devnode().and_then(|value| value.to_str()),
        )?;
        state.serialize_field(
            "subsystem_name",
            &self.0.subsystem().and_then(|value| value.to_str()),
        )?;
        state.serialize_field("system_name", &self.0.sysname().to_str())?;
        state.serialize_field("instance_number", &self.0.sysnum())?;
        state.serialize_field(
            "device_type",
            &self.0.devtype().and_then(|value| value.to_str()),
        )?;
        state.serialize_field("driver", &self.0.driver().and_then(|value| value.to_str()))?;
        state.serialize_field("action", &self.0.action().and_then(|value| value.to_str()))?;
        if let Some(parent) = &self.0.parent() {
            state.serialize_field("parent", &GenericSerializer::from(parent))?;
        } else {
            let x: Option<&str> = None;
            state.serialize_field("parent", &x)?;
        }
        state.serialize_field("properties", &DeviceUdevProperties::from(self.0))?;
        state.serialize_field("attributes", &DeviceUdevAttributes::from(self.0))?;
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
        .map(|device| serde_json::to_value(&GenericSerializer::from(&device)).unwrap())
        .collect();

    return vector;
}
