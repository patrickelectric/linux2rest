use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::SinkExt;
use paperclip::actix::Apiv2Schema;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::*;

#[derive(Clone, Serialize, PartialEq, Apiv2Schema)]
pub struct KernelMessage {
    facility: String,
    level: String,
    sequence_number: usize,
    timestamp_from_system_start_ns: u64,
    message: String,
}

impl KernelMessage {
    fn from_index_entry(index: usize, entry: &rmesg::entry::Entry) -> Self {
        KernelMessage {
            facility: match entry.facility {
                Some(facility) => facility.to_string(),
                None => "".into(),
            },
            level: match entry.level {
                Some(level) => level.to_string(),
                None => "".into(),
            },
            sequence_number: index,
            timestamp_from_system_start_ns: entry
                .timestamp_from_system_start
                .unwrap_or_default()
                .as_nanos() as u64,
            message: entry.message.clone(),
        }
    }
}

struct KernelService {
    messages: Vec<KernelMessage>,
    senders: Vec<Sender<String>>,
    main_loop_thread: std::thread::JoinHandle<()>,
}

lazy_static! {
    static ref KERNEL_SERVICE: Arc<Mutex<KernelService>> = Arc::new(Mutex::new(KernelService {
        messages: Default::default(),
        senders: Default::default(),
        main_loop_thread: thread::spawn(move || run_main_loop()),
    }));
}

pub fn ask_for_client() -> Receiver<String> {
    let (mut sender, receiver) = channel(10240);

    let mut kernel_service = KERNEL_SERVICE.as_ref().lock().unwrap();
    let _ = futures::executor::block_on(
        sender.send(serde_json::json!(&kernel_service.messages.clone()).to_string()),
    );
    kernel_service.senders.push(sender);

    return receiver;
}

fn add_message(message: KernelMessage) {
    let mut kernel_service = KERNEL_SERVICE.as_ref().lock().unwrap();
    kernel_service.messages.push(message.clone());

    kernel_service.senders.retain(|sender| {
        let mut sender = sender.clone();
        futures::executor::block_on(
            sender.send(serde_json::json!(&vec![message.clone()]).to_string()),
        )
        .is_ok()
    });
}

fn run_main_loop() {
    loop {
        let mut previous_entry: Option<rmesg::entry::Entry> = None;
        let mut index: usize = 0;
        for stream in rmesg::logs_iter(rmesg::Backend::Default, false, false) {
            for entry in stream {
                match entry {
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
                        let message = KernelMessage::from_index_entry(index, &previous_entry);
                        add_message(message);
                    }
                    Err(error) => {
                        warn!("Failed to parse kernel message {error}");
                    }
                };
            }
        }
    }
}

pub fn messages(start: Option<usize>, size: Option<usize>) -> Vec<KernelMessage> {
    let messages = KERNEL_SERVICE.as_ref().lock().unwrap().messages.clone();
    messages
        .iter()
        .skip(start.unwrap_or_default())
        .take(size.unwrap_or(messages.len()))
        .cloned()
        .collect()
}
