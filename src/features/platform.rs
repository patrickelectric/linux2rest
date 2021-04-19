use log::*;
use rppal;

pub fn generate_serde_value() -> serde_json::Value {
    match rppal::system::DeviceInfo::new() {
        Ok(system) => serde_json::json!({
            "model": system.model().to_string(),
            "soc": system.soc().to_string(),
        }),
        Err(error) => serde_json::json!({ "error": format!("{:?}", error) }),
    }
}
