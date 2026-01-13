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

    // Initialize Zenoh in background - don't block server startup
    tokio::spawn(async {
        loop {
            match zenoh::init().await {
                Ok(_) => {
                    info!("Zenoh initialized successfully");
                    break;
                }
                Err(error) => {
                    error!("Failed to initialize zenoh: {error}, retrying in 5 seconds...");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    });

    features::platform::start();
    recorder::start();
    server::run(&format!("0.0.0.0:{}", cli::args().as_ref().port));
}
