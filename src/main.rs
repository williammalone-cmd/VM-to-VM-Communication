// VM-to-VM-Communication

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::result;
use std::io::{self, Write, Read};
use std::thread;
use std::sync::{mpsc, Arc}; // Arc for sharing! Mpsc for messages!
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

type Result<T> = result::Result<T,()>;

// static SAFE_MODE: bool = false;
// static BAN_LIMIT: Duration = Duration::new(60* 10, 0);
static MESSAGE_RATE_LIMIT: Duration = Duration::from_millis(500);
// static MAX_STRIKES: u8 = 5;

#[derive(Debug)]
enum Message {
    ClientConnected(Arc<TcpStream>),
    ClientDisconnected(SocketAddr),
    NewMessage(SocketAddr, Vec<u8>),
}

struct Client {
    conn:Arc<TcpStream>,
    last_message: Instant,
    strike_count: u8,
}

fn client(stream: Arc<TcpStream>, messages: Sender<Message>) -> Result<()> {
    let addr = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(_) => {
            eprintln!("ERROR: Client connected but couldnt get address. Dropping ");
            return Err(());
        }
    };

    messages.send(Message::ClientConnected(stream.clone())).map_err(|err|{
        eprintln!("ERROR: Could not send ClientConnected: {}", err);
    })?;

    println!("INFO: Client thread started for {}", addr);
    let mut buffer = vec![0; 1024];

    loop {
        match (&*stream).read(&mut buffer){
            Ok(0) => {
                println!("INFO: Client {} disconnected gracefully", addr);
                messages.send(Message::ClientDisconnected(addr)).map_err(|_|{})?;
                break;
            }
            Ok(n) => {
                let data = buffer[0..n].to_vec();
                messages.send(Message::NewMessage(addr, data)).map_err(|err| {
                    eprintln!("ERROR: could not send NewMessage: {}", err);
                })?;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut => {
                continue;
            }
            Err(err) => {
                eprintln!("ERROR: reading from client {}: {}", addr, err);
                messages.send(Message::ClientDisconnected(addr)).map_err(|_|{})?;
                return Err(());
            }
        }
    }
    Ok(())
}

fn server(messages: Receiver<Message>) -> Result <()> {
    let mut clients: HashMap<SocketAddr, Client> = HashMap::new();
    let mut banned_clients: HashMap<IpAddr, Instant> = HashMap::new();

    println!("INFO: Server thread started.");

    loop {
        let msg = messages.recv().expect("ERROR: Message channel broken!");

        match msg {
            Message::ClientConnected(author_stream)=>{
                let author_addr = match author_stream.peer_addr() {
                    Ok(addr) => addr,
                    Err(_) => {
                        eprintln!("ERROR: Server received ClientConnected but couldnt get address");
                        let _ = author_stream.shutdown(std::net::Shutdown::Both);
                        continue;
                    }
                };

                let ip = author_addr.ip();
                let now = Instant::now();

                println!("INFO: Client accepted: {}", author_addr);
                clients.insert(author_addr, Client {
                    conn: author_stream,
                    last_message: now,
                    strike_count: 0,
                });
                
            } // end client connected

            Message::ClientDisconnected(author_addr) => {
                println!("INFO: Client Disconnected: {}", author_addr);
                clients.remove(&author_addr);
            }

            Message::NewMessage(author_addr, bytes) => {
                let now = Instant::now();
                let mut should_broadcast = true;

                if let Some(author) = clients.get_mut(&author_addr) {
                    // let time_since_last = now.duration_since(author.last_message);
                    author.last_message = now;
                    
                    // if time_since_last < MESSAGE_RATE_LIMIT {
                    //     author.strike_count = author.strike_count.saturating_add(1);
                    //     println!("INFO: Banning client {} for spamming", author_addr);
                    //     banned_clients.insert(author_addr.ip(), now);

                    //     let _ = (&*author.conn).write(b"You have been banned");
                    //     let _ = author.conn.shutdown(std::net::Shutdown::Both);

                    //     clients.remove(&author_addr);
                    //     continue;
                    // }else{
                        
                    //     author.strike_count = author.strike_count.saturating_sub(1);
                    // }
                } else {
                    eprintln!("WARN: Message from unknown client {}", author_addr);
                    should_broadcast = false;
                }

                if should_broadcast {
                    for (recipient_addr, recipient) in clients.iter(){
                        if *recipient_addr != author_addr {
                            let _= (&*recipient.conn).write(&bytes).map_err(|err| {
                                eprintln!("ERROR: Failed write to {}: {}", recipient_addr, err);
                            });
                        }
                    }
                }
            }
        }
    }
}

fn main() -> Result<()> {
    let address = "0.0.0.0:4567";
    let listener = TcpListener::bind(address)
        .map_err(|err|{eprintln!("ERROR: could not bind {}: {}", address, err);})?;
    println!("INFO: Listening on address: {}", address);
    
    let (sender, receiver) = mpsc::channel::<Message>();

    thread::spawn(|| server(receiver));

    for stream_result in listener.incoming() {
        match stream_result{
            Ok(stream) => {
                // println!("INFO: Incoming connection!");
                if let Err(e) = stream.set_read_timeout(Some(Duration::new(5, 0))) {
                    eprintln!("WARN: Failed to set read timeout: {}", e);
                }

                let message_sender = sender.clone();

                let stream_arc = Arc::new(stream);

                thread::spawn(move || {
                    if let Err(_) = client(stream_arc, message_sender){

                    }
                });
            }
            Err(e) => {
                eprintln!("ERROR: could not accept stream: {}", e);
            }
        }
    }
    
    Ok(())
}
