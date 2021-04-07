#[macro_use]
extern crate lazy_static;

mod cli;
mod features;
mod logger;

fn main() {
    logger::init();
    features::udev::generate_serde_value();
    features::system::generate_serde_value(features::system::SystemType::Components);
}
