use std::sync::Arc;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = env!("CARGO_PKG_NAME"),
    about = "Linux to REST API.",
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"))
]
pub struct Arguments {
    /// Activate debug/verbose mode
    #[structopt(short, long)]
    pub verbose: bool,

    /// Port to be used for REST API server
    #[structopt(long, default_value = "6030")]
    pub port: u16,
}

lazy_static! {
    static ref ARGS: Arc<Arguments> = Arc::new(Arguments::from_args());
}

pub fn args() -> Arc<Arguments> {
    ARGS.clone()
}
