use anyhow::{Context, Result};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::RwLock;
use std::sync::{mpsc, mpsc::channel};
use std::thread;
use ws::Message::*;

#[derive(Clone)]
struct Data {
    text: String,
    outward: ws::Sender,
}
lazy_static! {
    static ref DATA: RwLock<HashMap<i32, Data>> = RwLock::new(HashMap::new());
}
static NEXT_ID: AtomicI32 = AtomicI32::new(0);

fn main() -> Result<()> {
    let (user_msgs_tx, user_msgs_rx) = channel();
    // Start listener.
    thread::spawn(move || {
        ws::Builder::new()
            .with_settings(ws::Settings {
                tcp_nodelay: true,
                ..ws::Settings::default()
            })
            .build(|sender: ws::Sender| {
                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                // Add a state entry for the new user.
                DATA.write().unwrap().insert(
                    id,
                    Data {
                        text: "".to_string(),
                        outward: sender.clone(),
                    },
                );
                Conn {
                    id,
                    outward: sender,
                    inward: user_msgs_tx.clone(),
                }
            })
            .expect("when building")
            .listen("127.0.0.1:2794")
            .expect("when listening");
    });
    for msg in user_msgs_rx.iter() {
        match msg {
            Msg::SetText(id, ref text) => {
                // Update the user's text.
                let mut data = DATA.write().unwrap();
                let d = data[&id].clone();
                data.insert(
                    id,
                    Data {
                        text: text.to_string(),
                        ..d
                    },
                );
                // Tell everyone else.
                for (client_id, client) in data.iter() {
                    if client_id != &id {
                        client
                            .outward
                            .send(serde_json::to_string(&Msg::SetText(id, text.to_string()))?)?
                    };
                }
            }
            Msg::Remove(id) => {
                // Remove the user.
                let mut data = DATA.write().unwrap();
                data.remove(&id);
                // Tell everyone else.
                for (_, client) in data.iter() {
                    client
                        .outward
                        .send(serde_json::to_string(&Msg::Remove(id))?)?;
                }
            }
        }
        println!("{:?}", msg);
    }
    Ok(())
}

#[derive(Clone)]
struct Conn {
    pub id: i32,
    pub outward: ws::Sender,
    pub inward: mpsc::Sender<Msg>,
}

#[derive(Debug, Serialize, Deserialize)]
enum Msg {
    SetText(i32, String),
    Remove(i32),
}

impl ws::Handler for Conn {
    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        match msg {
            Text(t) => self.inward.send(Msg::SetText(self.id, t)).unwrap(),
            Binary(b) => println!("binary message: {:?}", b),
        };
        Ok(())
    }
    fn on_error(&mut self, e: ws::Error) {
        println!("err: {:?}", e);
    }
    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        let data = DATA.read().unwrap();
        // Tell the new user about everyone else.
        for (user, data) in data.iter() {
            if *user != self.id {
                self.outward.send(
                    serde_json::to_string(&Msg::SetText(*user, data.text.to_string())).unwrap(),
                )?
            }
        }
        Ok(())
    }
    fn on_close(&mut self, _code: ws::CloseCode, _reason: &str) {
        let mut data = DATA.write().unwrap();
        data.remove(&self.id);
        self.inward.send(Msg::Remove(self.id)).unwrap();
    }
}
