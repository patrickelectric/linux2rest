use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use log::*;
use serde::Deserialize;

use crate::features;

use super::websocket;

pub fn load_file(file_name: &str) -> String {
    // Load files at runtime only in debug builds
    if cfg!(debug_assertions) {
        use std::io::prelude::*;
        let html_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/html/");
        let mut file = std::fs::File::open(html_path.join(file_name)).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        return contents;
    }

    match file_name {
        "" | "index.html" => std::include_str!("../html/index.html").into(),
        "vue.js" => std::include_str!("../html/vue.js").into(),
        _ => format!("File not found: {}", file_name),
    }
}

pub fn root(req: HttpRequest) -> HttpResponse {
    let path = match req.match_info().query("filename") {
        "" => load_file("index.html"),
        file => load_file(file),
    };
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(path)
}

#[derive(Deserialize)]
pub struct KernelBufferQuery {
    start: Option<u64>,
    size: Option<u64>,
}

pub fn kernel_buffer(req: HttpRequest, query: web::Query<KernelBufferQuery>) -> HttpResponse {
    debug!("{:#?}", req);

    let query = query.into_inner();

    HttpResponse::Ok().content_type("application/json").body(
        serde_json::to_string_pretty(&features::kernel_buffer::generate_serde_value(
            query.start,
            query.size,
        ))
        .unwrap(),
    )
}

pub fn netstat(req: HttpRequest) -> HttpResponse {
    debug!("{:#?}", req);

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string_pretty(&features::netstat::generate_serde_value()).unwrap())
}

pub fn system(req: HttpRequest) -> HttpResponse {
    debug!("{:#?}", req);

    HttpResponse::Ok().content_type("application/json").body(
        serde_json::to_string_pretty(&features::system::generate_serde_value(
            features::system::SystemType::Everything,
        ))
        .unwrap(),
    )
}

pub fn udev(req: HttpRequest) -> HttpResponse {
    debug!("{:#?}", req);

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string_pretty(&features::udev::generate_serde_value()).unwrap())
}

pub fn platform(req: HttpRequest) -> HttpResponse {
    debug!("{:#?}", req);

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string_pretty(&features::platform::generate_serde_value()).unwrap())
}

pub fn websocket_kernel_buffer(req: HttpRequest, stream: web::Payload) -> HttpResponse {
    debug!("{:#?}", req);

    features::kernel_buffer::start_stream();

    ws::start(
        websocket::new_websocket(websocket::WebsocketEventType::KERNEL_BUFFER),
        &req,
        stream,
    )
    .unwrap_or_else(|error| {
        HttpResponse::BadRequest()
            .content_type("text/plain")
            .body(format!("error: {:#?}", error))
    })
}
