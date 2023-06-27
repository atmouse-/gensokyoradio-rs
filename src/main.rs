#[macro_use]
extern crate log;

use futures_util::{future, pin_mut, SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use std::{net::SocketAddr, time::Duration};
use std::sync::{Arc, Mutex};
use notify_rust::{Notification, Hint};
use std::sync::mpsc::channel;

use gensokyoradio::cache::CacheDir;
use gensokyoradio::message::{SongInfo, Greeting};
use gensokyoradio::session::Session;
use gensokyoradio::session::GensokyoMessage;

async fn handle_connection(ws_stream: TcpStream) -> Result<(), std::io::Error> {
    Ok(())
}

#[tokio::main]
async fn main() {
    let env = env_logger::Env::default()
        .filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    env_logger::init_from_env(env);

    let connect_addr = "wss://gensokyoradio.net/wss";

    let url = url::Url::parse(&connect_addr).unwrap();

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    info!("WebSocket handshake has been successfully completed");

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut interval = tokio::time::interval(Duration::from_millis(1000));

    let am_session = Arc::new(Mutex::new(
        Session::new()
    ));
    let am_cache_dir = Arc::new(Mutex::new(
        CacheDir::init()
    ));

    let (hint_tx, hint_rx) = channel::<SongInfo>();
    let hint_rx_monitor = Arc::new(Mutex::new(hint_rx));
    // 4 thread
    for n in 1..5 {
        let rx_monitor = hint_rx_monitor.clone();
        let am_cache_dir = am_cache_dir.clone();
        std::thread::spawn(move || loop {
            if let Ok(songinfo) = {
                let rx = rx_monitor.lock().unwrap();
                rx.recv()
            } {
                let cache_dir = am_cache_dir.lock().unwrap();
                let local_file = cache_dir.hash(&songinfo.albumart).unwrap().to_str()
                                    .unwrap()
                                    .to_owned();
                // TODO: unwrap
                Notification::new()
                    .summary(&songinfo.title)
                    .image_path(&local_file)
                    .body(&format!("{} / {}", &songinfo.artist, &songinfo.album))
                    //.icon("musicapp")
                    //.appname("musicapp")
                    //.hint(Hint::Category("email".to_owned()))
                    //.hint(Hint::Resident(true)) // this is not supported by all implementations
                    .timeout(&songinfo.remaining*1000) // this however is
                    .show()
                    ;
            }
        });
    }


    let greeting = Message::from(
        Greeting::default().to_json()
    );
    debug!("{}", &greeting);
    ws_sender.send(greeting).await.unwrap();
    ws_sender.flush().await.unwrap();

    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(msg) => {
                        let msg = msg.unwrap();

                        // skip binary message
                        if msg.is_binary() {break};
                        
                        //match GensokyoMessage::from
                        if msg.is_text() {
                            let am_session = am_session.clone();
                            let hint_tx = hint_tx.clone();
                            match GensokyoMessage::from(msg.to_text().unwrap()).unwrap() {
                                GensokyoMessage::MessageWelcome(m) => {
                                    debug!("{}", &m);
                                    let mut session = am_session.lock().unwrap();
                                    session.set_id(m.id);
                                },
                                GensokyoMessage::MessagePing(m) => {
                                    debug!("{}", &m);
                                    let session = am_session.lock().unwrap();
                                    let pong = session.gen_pong();
                                    debug!("{}", &pong);
                                    ws_sender.send(pong).await.unwrap();
                                    ws_sender.flush().await.unwrap();
                                },
                                GensokyoMessage::MessageSongInfo(m) => {
                                    info!("{:?}", &m);
                                    hint_tx.send(m.clone()).unwrap();
                                },
                                GensokyoMessage::MessageUnknown(m) => {
                                    debug!("unknow: {:?}", m);
                                },
                            }
                        }
                        
                        //    ws_sender.send(msg).await.unwrap();
                        //
                        if msg.is_close() {
                            break;
                        }
                    }
                    None => break,
                }
            }
            _ = interval.tick() => {
                ;
            }
        }
    }
    ;
}
