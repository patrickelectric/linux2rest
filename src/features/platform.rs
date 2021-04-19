use log::*;

pub fn generate_serde_value() -> serde_json::Value {
    #[cfg(feature = "raspberry")]
    {
        use rppal;
        return match rppal::system::DeviceInfo::new() {
            Ok(system) => serde_json::json!({
                "raspberry": {
                    "model": system.model().to_string(),
                    "soc": system.soc().to_string(),
                }
            }),
            Err(error) => serde_json::json!({ "error": format!("{:?}", error) }),
        };
    }

    #[cfg(not(feature = "raspberry"))]
    return serde_json::json!({
        "error": "Unsupported platform."
    });
}
