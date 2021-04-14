use actix_web::{HttpRequest, HttpResponse};
use log::*;

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
    HttpResponse::Ok().content_type("text/html").body(path)
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

pub fn raspberry(req: HttpRequest) -> HttpResponse {
    debug!("{:#?}", req);

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string_pretty(&features::raspberry::generate_serde_value()).unwrap())
}
