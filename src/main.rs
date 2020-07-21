use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use clap;

mod pages;

fn main() {
    let matches = clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("MAVLink to REST API!.")
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            clap::Arg::with_name("server")
                .short("s")
                .long("server")
                .value_name("IP:PORT")
                .help("Sets the IP and port that the rest server will be provided")
                .takes_value(true)
                .default_value("0.0.0.0:8088"),
        )
        .arg(
            clap::Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Be verbose")
                .takes_value(false),
        )
        .get_matches();

    let _verbose = matches.is_present("verbose");
    let server_string = matches.value_of("server").unwrap();

    println!("REST API address: {}", server_string);

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(pages::root_page))
            .route("/v4l", web::get().to(pages::v4l_page))
            //.route("/v4l", web::post().to(pages::control))
            .route("/v4l/interval", web::get().to(pages::interval))
    })
    .bind(server_string)
    .unwrap()
    .run()
    .unwrap();
}
