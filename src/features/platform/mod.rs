use cached::proc_macro::cached;
use paperclip::actix::Apiv2Schema;
use serde::Serialize;

#[cfg(feature = "raspberry")]
mod raspberry;

pub fn start() {
    #[cfg(feature = "raspberry")]
    raspberry::start_raspberry_events_scanner();
}

#[cfg(feature = "raspberry")]
#[derive(Clone, Serialize, Apiv2Schema)]
pub struct Raspberry {
    model: String,
    soc: String,
    events: raspberry::Events,
}

#[cfg(feature = "raspberry")]
#[derive(Clone, Serialize, Apiv2Schema)]
pub struct Platform {
    raspberry: Raspberry,
}

#[cfg(not(feature = "raspberry"))]
#[derive(Clone, Serialize, Apiv2Schema)]
pub struct Platform {}

#[cached(time = 5)]
pub fn platform() -> Result<Platform, String> {
    #[cfg(feature = "raspberry")]
    {
        use rppal;
        return match rppal::system::DeviceInfo::new() {
            Ok(system) => Ok(Platform {
                raspberry: Raspberry {
                    model: system.model().to_string(),
                    soc: system.soc().to_string(),
                    events: raspberry::events(),
                },
            }),
            Err(error) => Err(format!("{:?}", error)),
        };
    }

    #[cfg(not(feature = "raspberry"))]
    return Err(
        "Unsupported platform, make sure that platform is enabled during compilation time.".into(),
    );
}
