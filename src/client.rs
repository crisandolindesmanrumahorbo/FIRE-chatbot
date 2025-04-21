use std::error::Error;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct Client {}

impl Client {
    pub async fn perform(
        is_stream: bool,
        json_body: &str,
        mut client_stream: TcpStream,
    ) -> Result<(), Box<dyn Error>> {
        let request;
        if !is_stream {
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
            let mut llm_stream = TcpStream::connect("127.0.0.1:11434")
                .await
                .expect("error handshake");
            llm_stream
                .write_all(request.as_bytes())
                .await
                .expect("error write");

            let mut buffer = Vec::new();
            llm_stream
                .read_to_end(&mut buffer)
                .await
                .expect("Error read stream");

            let response = String::from_utf8_lossy(&buffer);
            println!("Receveived {}", response);
            client_stream
                .write_all(&buffer)
                .await
                .expect("Eerror chunk write");

            let _ = client_stream.flush().await;
        } else {
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
            let mut llm_stream = TcpStream::connect("127.0.0.1:11434")
                .await
                .expect("error handshake");
            println!("connected");
            llm_stream
                .write_all(request.as_bytes())
                .await
                .expect("error write");
            println!("written");

            let cors_headers = "HTTP/1.1 200 OK\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Content-Type: text/plain\r\n\
            Transfer-Encoding: chunked\r\n\
            \r\n";

            client_stream.write_all(cors_headers.as_bytes()).await?;

            let mut response_buffer = String::new();
            let mut full_collected = String::new();
            let mut chunk = [0u8; 4096];
            let mut json_buffer = Vec::new();
            let mut in_response = false;
            let mut escape_next = false;
            let mut response_bytes = Vec::new();
            loop {
                let bytes_read = match llm_stream.read(&mut chunk).await {
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
                                client_stream.write_all(chunk_header.as_bytes()).await?;
                                client_stream.write_all(text.as_bytes()).await?;
                                client_stream.write_all(b"\r\n").await?;
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
                            break;
                        }
                    }
                }
            }
        }
        // 4. Clean shutdown
        client_stream.shutdown().await?;
        println!("Connection closed properly");
        Ok(())
    }
}
