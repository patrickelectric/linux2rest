mod pages;
pub mod websocket;

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
            .route("/kernel_buffer", web::get().to(pages::kernel_buffer))
            .route("/netstat", web::get().to(pages::netstat))
            .route("/raspberry", web::get().to(pages::raspberry))
            .route("/system", web::get().to(pages::system))
            .route("/udev", web::get().to(pages::udev))
            .route(
                "/ws/kernel_buffer",
                web::get().to(pages::websocket_kernel_buffer),
            )
    })
    .bind(server_address)
    .unwrap()
    .run();

    let _ = system.run();
}
