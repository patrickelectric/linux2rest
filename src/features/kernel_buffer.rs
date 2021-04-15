use rmesg;

pub fn generate_serde_value(start: Option<u64>, size: Option<u64>) -> serde_json::Value {
    match rmesg::log_entries(rmesg::Backend::Default, true) {
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
