use std::io::Error;
use stockbit_chatbot::client::Client;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("localhost:8080")
        .await
        .expect("bind port error");
    println!("Server running on http://localhost:8080");

    loop {
        let (stream, _) = listener.accept().await.expect("error accept");
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream).await {
                eprintln!("connection error {}", e);
            }
        });
    }
}

async fn handle_client(mut stream: TcpStream) -> Result<(), Error> {
    let mut buffer = [0; 1024];
    // let size = stream.read(&mut buffer).await?;
    match stream.read(&mut buffer).await {
        Ok(size) => {
            let request = String::from_utf8_lossy(&buffer[..size]);
            if request.starts_with("OPTIONS") {
                let response = "HTTP/1.1 204 No Content\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Access-Control-Allow-Methods: POST, GET, OPTIONS\r\n\
            Access-Control-Allow-Headers: Content-Type\r\n\
            Access-Control-Max-Age: 86400\r\n\
            \r\n";

                stream.write_all(response.as_bytes()).await?;
                return Ok(());
            }
            if request.starts_with("POST /chatbot") {
                // let response_headers = "HTTP/1.1 200 OK\r\n\
                // Access-Control-Allow-Origin: *\r\n\
                // Content-Type: text/plain\r\n\
                // Transfer-Encoding: chunked\r\n\
                // \r\n";
                //
                // stream.write_all(response_headers.as_bytes()).await?;
                println!("request {}", request);
                let mut parts = request.split("\r\n\r\n");
                let _ = parts.next().expect("no req head ");
                let body = parts.next().expect("no body");
                Client::perform(true, body, stream).await;
            } else {
                stream
                    .write_all(
                        format!("{}{}", "HTTP/1.1 404 Not Found\r\n\r\n", "404 Not Found")
                            .as_bytes(),
                    )
                    .await
                    .expect("Error request url");
            }
        }
        Err(_) => {
            stream
                .write_all(
                    format!("{}{}", "HTTP/1.1 404 Not Found\r\n\r\n", "404 Not Found").as_bytes(),
                )
                .await
                .expect("Error request url");
        }
    }
    Ok(())
}
