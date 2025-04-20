use serde_json::json;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct Client {}

impl Client {
    pub async fn perform(is_stream: bool) {
        let json_body;
        let request;
        if !is_stream {
            json_body = json!({
                  "model": "gemma3:1b",
            "prompt": "hai"
                  })
            .to_string();
            request = format!(
                "POST /api/generate HTTP/1.1\r\n\
            Host: localhost:11434\r\n\
            Content-Type: application/json\r\n\
            Content-Length: {}\r\n\
            Connection: keep-alive\r\n\
            \r\n\
            {}",
                json_body.len(),
                json_body
            );
            let mut stream = TcpStream::connect("127.0.0.1:11434")
                .await
                .expect("error handshake");
            stream
                .write_all(request.as_bytes())
                .await
                .expect("error write");

            let mut buffer = Vec::new();
            stream
                .read_to_end(&mut buffer)
                .await
                .expect("Error read stream");

            let response = String::from_utf8_lossy(&buffer);
            println!("Receveived {}", response);
        } else {
            json_body = json!({
                  "model": "gemma3:1b",
            "prompt": "hai",
                 "stream": is_stream
                  })
            .to_string();

            request = format!(
                "POST /api/generate HTTP/1.1\r\n\
            Host: localhost:11434\r\n\
            Content-Type: application/json\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\
            \r\n\
            {}",
                json_body.len(),
                json_body
            );
            let mut stream = TcpStream::connect("127.0.0.1:11434")
                .await
                .expect("error handshake");
            stream
                .write_all(request.as_bytes())
                .await
                .expect("error write");

            // Process streaming response
            let mut buffer = [0; 1024];
            loop {
                let bytes_read = stream.read(&mut buffer).await.expect("Error read response");
                if bytes_read == 0 {
                    break;
                }

                let chunk = &buffer[..bytes_read];
                if let Ok(text) = std::str::from_utf8(chunk) {
                    println!("{}", text);
                }
            }
        }
    }
}
