use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;

#[tokio::main]
async fn main() {
    // Unwrap takes the value out of the result and panics if it is an error
    let listener = tokio::net::TcpListener::bind("localhost:5050").await.unwrap();
    let (tx, _rx) = broadcast::channel(10);
    loop {
        accept_client(&listener, &tx).await;
    }
}

async fn accept_client(listener: &TcpListener, tx: &Sender<(String, SocketAddr)>) {
    let (socket, addr) = listener.accept().await.unwrap();
    let tx = tx.clone();
    let rx = tx.subscribe();
    tokio::spawn(handle_message(socket, tx, rx, addr));
}

async fn handle_message(mut socket: TcpStream, tx: Sender<(String, SocketAddr)>, mut rx: broadcast::Receiver<(String, SocketAddr)>, addr: SocketAddr) {
    let (reader, mut write) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    loop {
        tokio::select! {
                result = reader.read_line(&mut line) => {
                    if result.unwrap() == 0 {
                        break;
                    }
                    tx.send((format!("{}: {}", addr, line), addr)).unwrap();
                    line.clear();
                }
                result = rx.recv() => {
                    let (msg, other_addr) = result.unwrap();
                        if addr != other_addr {
                            write.write_all(&msg.as_bytes()).await.unwrap();
                        }
                }
            }
    }
}
