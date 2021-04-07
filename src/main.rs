#[macro_use]
extern crate lazy_static;

mod cli;
mod features;
mod logger;

fn main() {
    logger::init();
    println!(
        "{}",
        serde_json::to_string_pretty(&features::udev::generate_serde_value()).unwrap()
    );
    features::system::generate_serde_value(features::system::SystemType::Components);
}
