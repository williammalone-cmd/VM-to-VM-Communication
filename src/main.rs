// VM-to-VM-Communication

use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use std::process::Command;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Arc;

const PORT: u16 = 4567;
const ADDRESS: &str = "0.0.0.0"; //Listen on every network on this machine

#[derive(Clone, Debug)]
struct ChatMsg {
    from: SocketAddr,
    text: String,
}

fn firewall(action: &str, port: u16){
    let arg = format!("--{}-port={}/tcp", action, port);
    match Command::new("firewall-cmd").arg(&arg).status(){
        Ok(s) if s.success() => eprintln!("INFO: Opened port {}", port),
        Ok(s) => eprintln!("ERROR: firewall failed {}", s),
        Err(e) => eprintln!("ERROR: could not run firewall cmd: {}", e),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut resolver: HashMap<String, String> = HashMap::new();
    resolver.insert("172.16.10.185".to_string(), "William".to_string()); 
    resolver.insert("172.16.10.194".to_string(), "Crystal".to_string());
    resolver.insert("172.16.10.192".to_string(), "Suhani".to_string());
    let resolver = Arc::new(resolver);

    let listener = TcpListener::bind((ADDRESS, PORT)).await?;
    let (tx, _) = broadcast::channel::<ChatMsg>(256);
    firewall("add", PORT);
    eprintln!("INFO: Listening on {}:{}\n", ADDRESS, PORT);

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            eprintln!("\nINFO: Shutting down...");
        }
        _ = async {
            loop {
                let (stream, addr) = match listener.accept().await{
                    Ok(conn) => conn,
                    Err(e) => {
                        eprintln!("accept failed: {}", e);
                        continue;
                    }
                };
                let tx = tx.clone(); 
                let resolver = resolver.clone();
                let mut rx = tx.subscribe();

                tokio::spawn(async move {
                    let Ok(ws) = accept_async(stream).await else{
                        eprintln!("{addr}: handshake failed");
                        return;
                    };
                    let (mut sink, mut source) = ws.split();
                    
                    let name = match resolver.get(&addr.ip().to_string())  {
                        Some(n) => n.clone(),
                        None => "Unknown User".to_string()
                    };

                    // Write loop
                    let write = tokio::spawn(async move {
                        while let Ok(msg) = rx.recv().await{
                            if msg.from == addr {
                                continue; // dont echo
                            }

                            let line = format!("[{}]: {}", name, msg.text);
                            if sink.send(Message::text(line)).await.is_err(){
                                break;
                            }
                        }
                    });

                    // Read Loop
                    while let Some(Ok(msg)) = source.next().await {
                        if let Message::Text(text) = msg {
                            // println!("text: {:?}", addr.to_string());
                            let packet = ChatMsg {
                                from: addr,
                                text: text.to_string(),
                            };
                            let _ = tx.send(packet);
                        }
                    }

                    write.abort();
                    eprintln!("INFO: {addr} disconnected");
                });
            }
        } => {}
    }

    firewall("remove", PORT);
    Ok(())
}