#[macro_use]
extern crate lazy_static;

mod cli;
mod features;
mod logger;

fn main() {
    logger::init();

    /*
    println!(
        "{}",
        serde_json::to_string_pretty(&features::udev::generate_serde_value()).unwrap()
    );
    */

    let mut i = 5;
    while i != 0 {
        features::system::generate_serde_value(features::system::SystemType::Everything);
        std::thread::sleep(std::time::Duration::from_secs(1));
        i -= 1;
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&features::system::generate_serde_value(
            features::system::SystemType::Everything
        ))
        .unwrap()
    );
}
