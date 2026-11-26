// Generic client skeleton

use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use rand::RngExt;

const ADDRESS: &str = "ws://172.16.10.185:4567";

async fn connect_with_backoff(url: &str) {
    let mut delay = Duration::from_secs(1);

    loop {
        match connect_async(url).await {
            Ok((ws, _)) => {
                delay = Duration::from_secs(1);
                let (mut sink, mut source) = ws.split();
                let _ = sink.send(Message::text("hello")).await;

                while let Some(Ok(msg)) = source.next().await {
                    eprintln!("recieved: {msg}");
                }
                eprintln!("disconnected, reconnecting...");
            }
            Err(e) => eprintln!("connect failed: {e}"),
        }
        let jitter = rand::rng().random_range(0..500);
        let wait = delay + Duration::from_millis(jitter);
        tokio::time::sleep(wait).await;
        delay = (delay*2).min(Duration::from_secs(30));
    }
}

#[tokio::main]
async fn main(){
    connect_with_backoff(ADDRESS).await;
}