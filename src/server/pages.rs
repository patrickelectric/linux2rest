use actix_web::{
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use actix_web_actors::ws;
use log::*;
use paperclip::actix::api_v2_operation;
use paperclip::actix::Apiv2Schema;
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

#[derive(Deserialize, Apiv2Schema)]
pub struct KernelBufferQuery {
    start: Option<u64>,
    size: Option<u64>,
}

#[api_v2_operation]
/// Provides kernel information, like dmesg
pub fn kernel_buffer(
    req: HttpRequest,
    query: web::Query<KernelBufferQuery>,
) -> Json<Vec<features::kernel_buffer::KernelMessage>> {
    debug!("{:#?}", req);

    let query = query.into_inner();

    Json(
        match features::kernel_buffer::messages(query.start, query.size) {
            Ok(content) => content,
            Err(error) => {
                debug!("{:?}", error);
                Vec::new()
            }
        },
    )
}

#[api_v2_operation]
/// Provides the same output as netstat: TCP/UDP ports that are in use and who is using it
pub fn netstat(req: HttpRequest) -> Json<features::netstat::Netstat> {
    debug!("{:#?}", req);

    Json(features::netstat::netstat())
}

#[api_v2_operation]
/// Provides system information: cpu, disk, operating system, memory, network, processes, sensors
pub async fn system(req: HttpRequest) -> Json<features::system::System> {
    debug!("{:#?}", req);

    Json(features::system::system())
}

#[api_v2_operation]
/// (WIP) Provides information about all devices connected to the main computer
pub fn udev(req: HttpRequest) -> HttpResponse {
    debug!("{:#?}", req);

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string_pretty(&features::udev::generate_serde_value()).unwrap())
}

#[api_v2_operation]
/// Provide platform specific information
pub async fn platform(req: HttpRequest) -> HttpResponse {
    debug!("{:#?}", req);

    match features::platform::platform() {
        Ok(content) => HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::to_string_pretty(&content).unwrap()),
        Err(error) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(format!("error: {}", error)),
    }
}

pub fn websocket_kernel_buffer(req: HttpRequest, stream: web::Payload) -> HttpResponse {
    debug!("{:#?}", req);

    features::kernel_buffer::start_stream();

    ws::start(
        websocket::new_websocket(websocket::WebsocketEventType::KernelBuffer),
        &req,
        stream,
    )
    .unwrap_or_else(|error| {
        HttpResponse::BadRequest()
            .content_type("text/plain")
            .body(format!("error: {:#?}", error))
    })
}
