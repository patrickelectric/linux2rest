#[macro_use]
extern crate lazy_static;

mod cli;
mod features;
mod logger;
mod recorder;
mod server;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    logger::init();

    features::system::start(std::time::Duration::from_secs(5));

    features::platform::start();
    recorder::start();
    server::run(&format!("0.0.0.0:{}", cli::args().as_ref().port));
}
