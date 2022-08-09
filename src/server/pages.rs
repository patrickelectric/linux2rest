use actix_web::{
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use actix_web_actors::ws;
use cached::proc_macro::cached;
use log::*;
use paperclip::actix::api_v2_operation;
use paperclip::actix::Apiv2Schema;
use serde::Deserialize;

use crate::features;

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
    start: Option<usize>,
    size: Option<usize>,
}

#[api_v2_operation]
/// Provides kernel information, like dmesg
pub fn kernel_buffer(
    req: HttpRequest,
    query: web::Query<KernelBufferQuery>,
) -> Json<Vec<features::kernel::KernelMessage>> {
    debug!("{:#?}", req);

    let query = query.into_inner();

    Json(features::kernel::messages(query.start, query.size))
}

#[cached(time = 10)]
#[api_v2_operation]
/// Provides the same output as netstat: TCP/UDP ports that are in use and who is using it
pub fn netstat(req: HttpRequest) -> Json<features::netstat::Netstat> {
    debug!("{:#?}", req);

    Json(features::netstat::netstat())
}

#[derive(Debug, Deserialize, Apiv2Schema)]
pub struct SerialQuery {
    udev: Option<bool>,
}

#[cached(time = 5)]
#[api_v2_operation]
/// Provides information about serial ports
pub async fn serial(
    req: HttpRequest,
    query: web::Query<SerialQuery>,
) -> Json<features::serial::SerialPorts> {
    debug!("{:#?}, {:#?}", req, &query);

    let query = query.into_inner();

    Json(features::serial::serial(query.udev))
}

#[cached(time = 5)]
#[api_v2_operation]
/// Provides system information: cpu, disk, operating system, memory, network, processes, sensors
pub async fn system(req: HttpRequest) -> Json<features::system::System> {
    debug!("{:#?}", req);

    Json(features::system::system())
}

#[cached(time = 1)]
#[api_v2_operation]
/// Provides system information for cpu only
pub async fn system_cpu(req: HttpRequest) -> Json<Vec<features::system::Cpu>> {
    debug!("{:#?}", req);

    Json(features::system::cpu())
}

#[cached(time = 1)]
#[api_v2_operation]
/// Provides system information for disk only
pub async fn system_disk(req: HttpRequest) -> Json<Vec<features::system::Disk>> {
    debug!("{:#?}", req);

    Json(features::system::disk())
}

#[cached(time = 1)]
#[api_v2_operation]
/// Provides system information from operating system only
pub async fn system_info(req: HttpRequest) -> Json<features::system::OsInfo> {
    debug!("{:#?}", req);

    Json(features::system::info())
}

#[cached(time = 5)]
#[api_v2_operation]
/// Provides system information for memory only
pub async fn system_memory(req: HttpRequest) -> Json<features::system::Memory> {
    debug!("{:#?}", req);

    Json(features::system::memory())
}

#[cached(time = 5)]
#[api_v2_operation]
/// Provides system information for network only
pub async fn system_network(req: HttpRequest) -> Json<Vec<features::system::Network>> {
    debug!("{:#?}", req);

    Json(features::system::network())
}

#[cached(time = 5)]
#[api_v2_operation]
/// Provides system information for processes only
pub async fn system_process(req: HttpRequest) -> Json<Vec<features::system::Process>> {
    debug!("{:#?}", req);

    Json(features::system::process())
}

#[cached(time = 5)]
#[api_v2_operation]
/// Provides system information for sensors only
pub async fn system_temperature(req: HttpRequest) -> Json<Vec<features::system::Temperature>> {
    debug!("{:#?}", req);

    Json(features::system::temperature())
}

#[api_v2_operation]
/// Provides system information about current unix time
pub async fn system_unix_time_seconds(req: HttpRequest) -> HttpResponse {
    debug!("{:#?}", req);

    HttpResponse::Ok()
        .content_type("text/plain")
        .body(features::system::unix_time_seconds().to_string())
}

#[cached(time = 10)]
#[api_v2_operation]
/// (WIP) Provides information about all devices connected to the main computer
pub fn udev(req: HttpRequest) -> HttpResponse {
    debug!("{:#?}", req);

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string_pretty(&features::udev::generate_serde_value()).unwrap())
}

#[cached(time = 5)]
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

    ws::start(
        features::kernel_websocket::new_websocket(
            features::kernel_websocket::WebsocketEventType::KernelBuffer,
        ),
        &req,
        stream,
    )
    .unwrap_or_else(|error| {
        HttpResponse::BadRequest()
            .content_type("text/plain")
            .body(format!("error: {:#?}", error))
    })
}
