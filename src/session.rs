use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use chrono::{DateTime, Utc};

use super::message::SongInfo;
use super::message::Greeting;
use super::message::Welcome;
use super::message::Ping;
use super::message::Pong;

impl SongInfo {
    pub fn from(data: &str) -> Result<SongInfo, serde_json::Error> {
        serde_json::from_str::<SongInfo>(data)
    }
}

impl Welcome {
    pub fn from(data: &str) -> Result<Welcome, ()> {
        if let Ok(welcome) = serde_json::from_str::<Welcome>(data) {
            return match welcome.message.as_str() {
                "welcome" => Ok(welcome),
                _ => Err(()),
            }
        };
        Err(())
    }
}

impl Greeting {
    pub fn default() -> Greeting {
        let mut greeting = Greeting::new();
        greeting.message = String::from("grInitialConnection");
        greeting
    }

    pub fn from(data: &str) -> Result<Greeting, ()> {
        if let Ok(greeting) = serde_json::from_str::<Greeting>(data) {
            return match greeting.message.as_str() {
                "greeting" => Ok(greeting),
                _ => Err(()),
            }
        };
        Err(())
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl Ping {
    pub fn from(data: &str) -> Result<Ping, ()> {
        if let Ok(ping) = serde_json::from_str::<Ping>(data) {
            return match ping.message.as_str() {
                "ping" => Ok(ping),
                _ => Err(()),
            }
        };
        Err(())
    }
}

impl Pong {
    pub fn default(id: i64) -> Pong {
        let mut pong = Pong::new();
        pong.message = String::from("pong");
        pong.id = id;
        pong
    }

    pub fn from(data: &str) -> Result<Pong, ()> {
        if let Ok(pong) = serde_json::from_str::<Pong>(data) {
            return match pong.message.as_str() {
                "pong" => Ok(pong),
                _ => Err(()),
            }
        };
        Err(())
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

pub enum GensokyoMessage {
    MessageUnknown(String),
    MessageWelcome(Welcome),
    MessageSongInfo(SongInfo),
    MessagePing(Ping),
}

impl GensokyoMessage {
    pub fn from(data: &str) -> Result<GensokyoMessage, std::io::Error> {
        if let Ok(ping) = Ping::from(data) {
            return Ok(GensokyoMessage::MessagePing(ping))
        }
        if let Ok(songinfo) = SongInfo::from(data) {
            return Ok(GensokyoMessage::MessageSongInfo(songinfo))
        }
        if let Ok(welcome) = Welcome::from(data) {
            return Ok(GensokyoMessage::MessageWelcome(welcome))
        }
        Ok(GensokyoMessage::MessageUnknown(data.to_owned()))
    }
}

pub struct Session {
    id: i64,
    start_time: DateTime<Utc>,
}

impl Session {
    pub fn new() -> Session {
        Session {
            id: 0,
            start_time: Utc::now(),
        }
    }

    pub fn from_welcome(welcome: Welcome) -> Session {
        Session {
            id: welcome.id,
            start_time: Utc::now(),
        }
    }

    pub fn set_id(&mut self, id: i64) -> () {
        self.id = id
    }

    pub fn gen_pong(&self) -> Message {
        let pong = Pong::default(self.id).to_json();
        Message::from(pong)
    }
}
