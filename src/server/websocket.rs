use actix::{self, Actor, Addr, AsyncContext, Handler, Message, StreamHandler};
use actix_web_actors::ws;
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

#[derive(Debug)]
pub struct WebsocketActorContent {
    pub actor: Addr<WebsocketActor>,
    pub event_type: WebsocketEventType,
}

#[derive(Debug, Default)]
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

impl WebsocketManager {
    pub fn send(&self, event_type: WebsocketEventType, value: &serde_json::Value) {
        if self.clients.is_empty() {
            return;
        }

        let string = serde_json::to_string_pretty(value).unwrap();
        self.clients
            .iter()
            .filter(|client| client.event_type == event_type)
            .for_each(|client| {
                client.actor.do_send(StringMessage(string.clone()));
            });
    }
}

pub fn new_websocket(event_type: WebsocketEventType) -> WebsocketActor {
    WebsocketActor::new(SYSTEM.clone(), event_type)
}

#[derive(Debug)]
pub struct WebsocketActor {
    server: Arc<Mutex<WebsocketManager>>,
    event_type: WebsocketEventType,
}

impl WebsocketActor {
    pub fn new(server: Arc<Mutex<WebsocketManager>>, event_type: WebsocketEventType) -> Self {
        Self { server, event_type }
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
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebsocketActor {
    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("Starting websocket, add itself in manager.");
        self.server
            .lock()
            .unwrap()
            .clients
            .push(WebsocketActorContent {
                actor: ctx.address(),
                event_type: self.event_type,
            });
    }

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
