use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};

use rscam::{Camera, Control, CtrlData, Settable};
use serde;
use serde_json::json;
use std::{fs, mem};

pub fn root_page(_req: HttpRequest) -> impl Responder {
    format!("Hello!")
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CameraIntervalRequest {
    camera: String,
    fourcc: [u8; 4],
    resolution: (u32, u32),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Value {
    Integer(i64),
    Bool(bool),
    String(String),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ControlValue {
    id: u32,
    value: Value,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ConfigureCamera {
    camera: String,
    control: ControlValue
}

pub fn interval(_req: HttpRequest, json: web::Json<CameraIntervalRequest>) -> impl Responder {
    let intervalRequest = json.into_inner();

    let camera = Camera::new(&intervalRequest.camera).unwrap();
    let intervals = camera.intervals(intervalRequest.fourcc, intervalRequest.resolution);
    let intervals = intervals.unwrap();
    serde_json::to_string_pretty(&intervals)
}

pub fn v4l_page(_req: HttpRequest) -> impl Responder {
    let cameras_path = fs::read_dir("/dev/")
        .unwrap()
        .filter(|f| {
            f.as_ref()
                .unwrap()
                .file_name()
                .to_str()
                .unwrap()
                .starts_with("video")
        })
        .map(|f| String::from(f.unwrap().path().clone().to_str().unwrap()))
        .collect::<Vec<_>>();

    let mut main_json = json!({"cameras":{}});
    let cameras_json = &mut main_json["cameras"];
    'camera_loop: for camera_path in cameras_path {
        let camera = Camera::new(&camera_path);
        if let Err(error) = camera {
            print!("Error while opening camera: {:#?}", error);
            continue;
        }
        let camera = camera.unwrap();

        // Get camera control
        let mut controls = Vec::<serde_json::value::Value>::new();
        for control in camera.controls() {
            match &control {
                Ok(control) => {
                    controls.push(serde_json::to_value(control).unwrap());
                }
                Err(error) => {
                    print!("Error while fetching control: {:#?}", error);
                    continue 'camera_loop;
                }
            }
        }
        cameras_json[&camera_path]["controls"] = serde_json::to_value(controls).unwrap();

        // Get camera format
        let mut formats = Vec::<serde_json::value::Value>::new();
        for format in camera.formats() {
            match format {
                Ok(format) => {
                    let mut values = serde_json::to_value(&format).unwrap();
                    let resolutions = camera.resolutions(format.format).unwrap();
                    values["resolutions"] = serde_json::to_value(resolutions).unwrap();
                    formats.push(values);
                }
                Err(error) => {
                    print!("Error while fetching video format: {:#?}", error);
                    continue;
                }
            }
        }
        cameras_json[&camera_path]["formats"] = serde_json::to_value(formats).unwrap();
    }
    serde_json::to_string_pretty(&main_json)
}


pub fn control(req: HttpRequest, json: web::Json<ConfigureCamera>) {
    let configuration = json.into_inner();
    let camera = Camera::new(&configuration.camera).unwrap();

    let value = match &configuration.control.value {
        Value::Integer(value) => {
            Settable::unify(value)
        },
        Value::Bool(value) => {
            Settable::unify(value)
        },
        Value::String(value) => {
            Settable::unify(value)
        },
    };

    camera.set_control(configuration.control.id, &value);
}
