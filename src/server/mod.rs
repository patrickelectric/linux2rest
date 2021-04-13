mod pages;

use actix_web::{rt::System, web, App, HttpServer};

// Start REST API server with the desired address
pub fn run(server_address: &str) {
    let server_address = server_address.to_string();

    // Start HTTP server thread
    let system = System::new("http-server");
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(pages::root))
            .route(
                r"/{filename:.*(\.html|\.js|\.css)}",
                web::get().to(pages::root),
            )
            .route("/netstat", web::get().to(pages::netstat))
            .route("/system", web::get().to(pages::system))
            .route("/udev", web::get().to(pages::udev))
    })
    .bind(server_address)
    .unwrap()
    .run();

    let _ = system.run();
}
