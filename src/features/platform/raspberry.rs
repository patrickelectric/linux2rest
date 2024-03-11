use std::sync::{mpsc::channel, Arc, Mutex, Once};

use chrono;
use tracing::*;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use paperclip::actix::Apiv2Schema;
use serde::Serialize;

#[derive(Clone, Serialize, Apiv2Schema)]
pub enum EventType {
    FrequencyCapping,
    TemperatureLimit,
    Throttling,
    UnderVoltage,
}

#[derive(Clone, Serialize, Apiv2Schema)]
pub struct Event {
    pub time: chrono::DateTime<chrono::Local>,
    #[serde(rename = "type")]
    pub typ: EventType,
}

#[derive(Clone, Default, Serialize, Apiv2Schema)]
pub struct Events {
    pub occurring: Vec<Event>,
    pub list: Vec<Event>,
}

static ONCE: Once = Once::new();

lazy_static! {
    static ref EVENTS: Arc<Mutex<Events>> = Arc::new(Mutex::new(Events::default()));
}

pub fn events() -> Events {
    return EVENTS.lock().unwrap().clone();
}

pub fn start_raspberry_events_scanner() {
    ONCE.call_once(|| {
        let event_file = "/sys/devices/platform/soc/soc:firmware/get_throttled";

        std::thread::spawn(move || {
            let (tx, rx) = channel();
            let mut watcher: RecommendedWatcher = Watcher::new_raw(tx).unwrap();
            watcher
                .watch(event_file, RecursiveMode::NonRecursive)
                .unwrap();

            loop {
                match rx.recv() {
                    Ok(event) => {
                        debug!("Event in: {}, reason: {:#?}", event_file, event);
                        let content = std::fs::read_to_string(event_file)
                            .expect("Something went wrong reading the file");
                        let firmware_event = u64::from_str_radix(content.trim(), 16).unwrap();

                        let time = chrono::Local::now();
                        let mut occurring_events = vec![];
                        if (firmware_event & 0b1) != 0 {
                            occurring_events.push(Event {
                                time: time,
                                typ: EventType::UnderVoltage,
                            });
                        }

                        if (firmware_event & 0b10) != 0 {
                            occurring_events.push(Event {
                                time: time,
                                typ: EventType::FrequencyCapping,
                            });
                        }

                        if (firmware_event & 0b100) != 0 {
                            occurring_events.push(Event {
                                time: time,
                                typ: EventType::Throttling,
                            });
                        }

                        if (firmware_event & 0b1000) != 0 {
                            occurring_events.push(Event {
                                time: time,
                                typ: EventType::TemperatureLimit,
                            });
                        }

                        let mut events = EVENTS.lock().unwrap();
                        events.occurring = occurring_events.clone();
                        events.list.extend(occurring_events);
                    }
                    Err(error) => warn!("Failed to watch: {}, reason: {:?}", event_file, error),
                }
            }
        });
    });
}
