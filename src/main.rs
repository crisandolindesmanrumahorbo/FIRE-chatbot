use std::io::Error;
use stockbit_chatbot::client::Client;
use tokio::net::{TcpListener, TcpSocket, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let is_stream = true;
    println!("Hello, world!");
    Client::perform(is_stream).await;
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("bind port error");

    let (stream, _) = listener.accept().await.expect("error accept");

    tokio::spawn(async move {
        if let Err(e) = handle_client(stream).await {
            eprintln!("connection error {}", e);
        }
    });

    Ok(())
}

async fn handle_client(stream: TcpStream) -> Result<(), Error> {
    Ok(())
}
