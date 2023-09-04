use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;

/// Entry point for the application.
#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "5050".to_string());
    let address = &format!("localhost:{}", port);
    let listener = TcpListener::bind(address).await.expect(&format!("Failed to bind to port {}", port));
    println!("Listening on {}", address);

    // Create a broadcast channel with a capacity of 10 messages.
    let (tx, _rx) = broadcast::channel(10);

    // Loop to continually accept new clients.
    loop {
        if let Err(e) = accept_client(&listener, &tx).await {
            println!("Failed to accept client: {}", e);
        }
    }
}

/// Accepts a new client and spawns a task to handle it.
async fn accept_client(listener: &TcpListener, tx: &Sender<(String, SocketAddr)>) -> Result<(), Box<dyn std::error::Error>> {
    let (socket, addr) = listener.accept().await?;

    println!("Accepted client: {}", addr);
    let tx = tx.clone();
    let rx = tx.subscribe();
    tokio::spawn(handle_messages(socket, tx, rx, addr));
    Ok(())
}

/// Handles the connected client.
async fn handle_messages(
    mut socket: TcpStream,
    tx: Sender<(String, SocketAddr)>,
    mut rx: broadcast::Receiver<(String, SocketAddr)>,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    // Split the socket into a read and a write half.
    let (reader, mut write) = socket.split();

    // Create a buffered reader for the read half.
    let mut reader = BufReader::new(reader);

    // Initialize a string to store messages.
    let mut line = String::new();

    // Loop to read and write messages.
    loop {
        tokio::select! {
            // Read a line from the client and broadcast it.
            result = reader.read_line(&mut line) => {
                if let Ok(bytes_read) = result {
                    if bytes_read == 0 {
                        break;
                    }
                    tx.send((format!("{}: {}", addr, line), addr))?;
                    line.clear();
                } else {
                    return Err("Error reading from client".into());
                }
            }
            // Receive a message from the channel and write it to the client.
            result = rx.recv() => {
                if let Ok((msg, other_addr)) = result {
                    if addr != other_addr {
                        write.write_all(msg.as_bytes()).await?;
                    }
                } else {
                    return Err("Error receiving broadcast".into());
                }
            }
        }
    }
    println!("Client {} disconnected", addr);
    Ok(())
}
