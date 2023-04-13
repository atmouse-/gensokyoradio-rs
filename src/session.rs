use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use chrono::{DateTime, Utc};

use super::message::SongInfo;

pub struct Welcome {
    pub id: i64,
}

impl Welcome {
    pub fn from(data: &str) -> Result<Welcome, ()> {
        match data.strip_prefix("welcome:") {
            Some(value) => {
                match value.parse::<i64>() {
                    Ok(id) => Ok(
                        Welcome {
                            id: id,
                        }
                    ),
                    Err(_) => {Err(())}, // welcome protocol error
                }
                
            },
            None => {Err(())}
        }
    }
}

pub struct Ping;

impl Ping {
    pub fn from(data: &str) -> Result<Ping, ()> {
        if data.eq("ping") {
            Ok(Ping)
        } else {
            Err(())
        }
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
        if let Ok(songinfo) = serde_json::from_str::<SongInfo>(data) {
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

    pub fn gen_pong(&self) -> Result<Message, ()> {
        Ok(Message::from(
            format!("pong:{}", self.id)
        ))
    }
}
