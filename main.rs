use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    // Unwrap takes the value out of the result and panics if it is an error
    let listener = tokio::net::TcpListener::bind("localhost:5050").await.unwrap();
    let(mut socket, _addr) = listener.accept().await.unwrap();
    let mut buffer = [0u8; 1024];
    // The number of bytes read is returned so that we can write exactly that many bytes back.
    let bytes_read = socket.read(&mut buffer).await.unwrap();

    socket.write_all(&buffer[..bytes_read]).await.unwrap();
}
