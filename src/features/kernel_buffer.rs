use crate::server::websocket;

use rmesg;

use std::sync::Once;

static ONCE: Once = Once::new();

pub fn start_stream() {
    ONCE.call_once(|| {
        std::thread::spawn(|| loop {
            let mut previous_entry: Option<rmesg::entry::Entry> = None;
            let mut index: u64 = 0;
            for stream in rmesg::logs_iter(rmesg::Backend::Default, false, false) {
                for entry in stream {
                    let message = match entry {
                        Ok(entry) => {
                            if entry.facility.is_some() {
                                if previous_entry.is_some() {
                                    index += 1;
                                }
                                previous_entry = Some(entry.clone());
                            }

                            if previous_entry.is_none() {
                                continue;
                            }

                            let previous_entry = previous_entry.clone().unwrap();
                            serde_json::json!({
                                "facility": previous_entry.facility,
                                "level": previous_entry.level,
                                "sequence_number": index,
                                "timestamp_from_system_start_ns": previous_entry.timestamp_from_system_start.unwrap_or_default().as_nanos() as u64,
                                "message": entry.message,
                            })
                        }
                        Err(error) => {
                            serde_json::json!({
                                "error": format!("{:#?}", error),
                            })
                        }
                    };

                    websocket::manager().lock().unwrap().send(websocket::WebsocketEventType::KERNEL_BUFFER, &message);
                }
            }
        });
    });
}

pub fn generate_serde_value(start: Option<u64>, size: Option<u64>) -> serde_json::Value {
    match rmesg::log_entries(rmesg::Backend::Default, false) {
        Ok(entries) => {
            let mut entries: Vec<rmesg::entry::Entry> = entries.into();

            let mut mut_iter = entries.iter_mut();
            let mut last_valid_iter = mut_iter.next().unwrap();
            while let Some(entry) = mut_iter.next() {
                if entry.sequence_num.is_some() {
                    last_valid_iter = entry;
                    continue;
                }

                last_valid_iter.message += &format!("\n{}", entry.message);
            }

            let entries = entries
                .iter()
                .filter(|entry| entry.sequence_num.is_some())
                .enumerate()
                .skip(start.unwrap_or_default() as usize);

            let length = entries.size_hint().1.unwrap() as u64;
            let entries: Vec<serde_json::Value>  = entries
                .take(size.unwrap_or(length) as usize)
                .map(|(index, entry)| {
                    serde_json::json!({
                        "facility": entry.facility,
                        "level": entry.level,
                        "sequence_number": index,
                        "timestamp_from_system_start_ns": entry.timestamp_from_system_start.unwrap_or_default().as_nanos() as u64,
                        "message": entry.message,
                    })
                })
                .collect();
            return serde_json::to_value(&entries).unwrap();
        }
        Err(error) => {
            return serde_json::json!({ "error": format!("{:?}", error) });
        }
    };
}
