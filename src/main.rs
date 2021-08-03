#[macro_use]
extern crate lazy_static;

mod cli;
mod features;
mod logger;
mod server;

fn main() {
    logger::init();
    features::platform::start();

    server::run(&format!("0.0.0.0:{}", cli::args().as_ref().port));
}
