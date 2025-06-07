use tokio::{
    io::{AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};

use crate::cfg::get_config;

pub struct LlamaClient {
    stream: TcpStream,
}

impl LlamaClient {
    pub async fn new() -> Self {
        Self {
            stream: TcpStream::connect(get_config().llama_url)
                .await
                .expect("error handshake"),
        }
    }

    pub async fn send(&mut self, json_body: &str) {
        let request = format!(
            "POST /api/generate HTTP/1.1\r\n\
            Host: {}\r\n\
            Content-Type: application/json\r\n\
            Content-Length: {}\r\n\
            Connection: keep-alive\r\n\
            \r\n\
            {}",
            get_config().llama_url,
            json_body.len(),
            json_body
        );

        self.stream
            .write_all(request.as_bytes())
            .await
            .expect("error write");
    }

    pub async fn stream_response(&mut self, mut client_writer: impl AsyncWrite + Unpin) -> String {
        let cors_headers = "HTTP/1.1 200 OK\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Content-Type: text/plain\r\n\
            Transfer-Encoding: chunked\r\n\
            \r\n";

        client_writer
            .write_all(cors_headers.as_bytes())
            .await
            .expect("error header");

        let mut response_buffer = String::new();
        let mut full_collected = String::new();
        let mut chunk = [0u8; 4096];
        let mut json_buffer = Vec::new();
        let mut in_response = false;
        let mut escape_next = false;
        let mut response_bytes = Vec::new();
        loop {
            let bytes_read = match self.stream.read(&mut chunk).await {
                Ok(0) => {
                    println!("Ollama connection closed");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    break;
                }
            };

            for &byte in &chunk[..bytes_read] {
                if in_response {
                    if escape_next {
                        // Handle escaped characters
                        response_bytes.push(match byte {
                            b'n' => b'\n',
                            b'r' => b'\r',
                            b't' => b'\t',
                            b'0' => b'\0',
                            _ => byte, // Includes \" and \\
                        });
                        escape_next = false;
                    } else if byte == b'\\' {
                        escape_next = true;
                    } else if byte == b'"' {
                        // End of response value
                        in_response = false;
                        if let Ok(text) = String::from_utf8(response_bytes.clone()) {
                            response_buffer.push_str(&text);
                            full_collected.push_str(&text);

                            // Send chunk with just the response text
                            let chunk_header = format!("{:x}\r\n", text.len());
                            client_writer
                                .write_all(chunk_header.as_bytes())
                                .await
                                .expect("error");
                            client_writer.write_all(text.as_bytes()).await.expect("");
                            client_writer.write_all(b"\r\n").await.expect("");
                        }
                        response_bytes.clear();
                    } else {
                        response_bytes.push(byte);
                    }
                } else {
                    // Look for "response":" pattern
                    json_buffer.push(byte);
                    if json_buffer.ends_with(b"\"response\":\"") {
                        in_response = true;
                        json_buffer.clear();
                    }

                    // Check for stream end
                    if json_buffer.ends_with(b"\"done\":true") {
                        println!("done");
                        self.stream.shutdown().await.expect("shutdown");
                        println!("Connection closed properly");
                        return full_collected;
                    }
                }
            }
        }
        full_collected
    }
}
