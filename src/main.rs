#[macro_use]
extern crate lazy_static;

mod cli;
mod features;
mod logger;
mod recorder;
mod server;
mod zenoh;

use tracing::*;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    logger::init();

    features::system::start(std::time::Duration::from_secs(5));

    while let Err(error) = zenoh::init().await {
        error!("Failed to initialize zenoh: {error}");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }

    features::platform::start();
    recorder::start();
    server::run(&format!("0.0.0.0:{}", cli::args().as_ref().port));
}
