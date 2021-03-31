#[macro_use]
extern crate lazy_static;

mod cli;
mod logger;

fn main() {
    logger::init();
}
