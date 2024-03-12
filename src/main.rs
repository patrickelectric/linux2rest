#[macro_use]
extern crate lazy_static;

mod cli;
mod features;
mod logger;
mod recorder;
mod server;

fn main() {
    logger::init();
    features::platform::start();
    recorder::start();
    server::run(&format!("0.0.0.0:{}", cli::args().as_ref().port));
}
