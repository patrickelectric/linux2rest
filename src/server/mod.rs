mod pages;

use actix_web::{rt::System, App, HttpServer};
use paperclip::actix::{web, OpenApiExt};

// Start REST API server with the desired address
pub fn run(server_address: &str) {
    let server_address = server_address.to_string();

    // Start HTTP server thread
    let system = System::new("http-server");
    HttpServer::new(|| {
        App::new()
            .wrap_api()
            .with_json_spec_at("/docs.json")
            .with_swagger_ui_at("/docs")
            .route("/", web::get().to(pages::root))
            .route(
                r"/{filename:.*(\.html|\.js|\.css)}",
                web::get().to(pages::root),
            )
            .route("/kernel_buffer", web::get().to(pages::kernel_buffer))
            .route("/netstat", web::get().to(pages::netstat))
            .route("/platform", web::get().to(pages::platform))
            .route("/system", web::get().to(pages::system))
            .route("/system/cpu", web::get().to(pages::system_cpu))
            .route("/system/disk", web::get().to(pages::system_disk))
            .route("/system/info", web::get().to(pages::system_info))
            .route("/system/memory", web::get().to(pages::system_memory))
            .route("/system/network", web::get().to(pages::system_network))
            .route("/system/process", web::get().to(pages::system_process))
            .route(
                "/system/temperature",
                web::get().to(pages::system_temperature),
            )
            .route(
                "/system/unix_time_seconds",
                web::get().to(pages::system_unix_time_seconds),
            )
            .route("/udev", web::get().to(pages::udev))
            .route(
                "/ws/kernel_buffer",
                web::get().to(pages::websocket_kernel_buffer),
            )
            .build()
    })
    .bind(server_address)
    .unwrap()
    .run();

    let _ = system.run();
}
