use crate::features::kernel;
use actix::prelude::*;
use actix::{self, Actor, Addr, AsyncContext, Handler, Message, StreamHandler};
use actix_web_actors::ws;
use futures::channel::mpsc::Receiver;
use serde::Serialize;

use log::*;

use std::sync::{Arc, Mutex};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum WebsocketEventType {
    KernelBuffer,
}

pub struct StringMessage(String);

impl Message for StringMessage {
    type Result = ();
}

#[derive(Serialize, Debug)]
pub struct WebsocketError {
    pub error: String,
}

pub struct WebsocketActorContent {
    pub actor: Addr<WebsocketActor>,
    pub event_type: WebsocketEventType,
}

#[derive(Default)]
pub struct WebsocketManager {
    pub clients: Vec<WebsocketActorContent>,
}

lazy_static! {
    static ref SYSTEM: Arc<Mutex<WebsocketManager>> =
        Arc::new(Mutex::new(WebsocketManager::default()));
}

pub fn manager() -> Arc<Mutex<WebsocketManager>> {
    return SYSTEM.clone();
}

pub fn new_websocket(_event_type: WebsocketEventType) -> WebsocketActor {
    WebsocketActor::new(SYSTEM.clone())
}

pub struct WebsocketActor {
    server: Arc<Mutex<WebsocketManager>>,
    receiver: Option<Receiver<String>>,
}

impl WebsocketActor {
    pub fn new(server: Arc<Mutex<WebsocketManager>>) -> Self {
        Self {
            server,
            receiver: Some(kernel::ask_for_client()),
        }
    }
}

impl Handler<StringMessage> for WebsocketActor {
    type Result = ();

    fn handle(&mut self, message: StringMessage, context: &mut Self::Context) {
        context.text(message.0);
    }
}

impl Actor for WebsocketActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("Starting websocket");
        ctx.add_stream(self.receiver.take().unwrap());
    }
}

impl StreamHandler<String> for WebsocketActor {
    fn handle(&mut self, data: String, ctx: &mut Self::Context) {
        ctx.text(data)
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebsocketActor {
    fn finished(&mut self, ctx: &mut Self::Context) {
        debug!("Finishing websocket, remove itself from manager.");
        self.server
            .lock()
            .unwrap()
            .clients
            .retain(|x| x.actor != ctx.address());
    }

    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(_)) => {
                ctx.text(
                    serde_json::to_string(&WebsocketError {
                        error: "Websocket does not support inputs.".to_string(),
                    })
                    .unwrap(),
                );
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}
