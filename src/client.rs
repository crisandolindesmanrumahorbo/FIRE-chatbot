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

            // 1. Read Ollama's headers first
            let mut ollama_headers = Vec::new();
            let mut header_ended = false;
            let mut buf = [0u8; 1];

            // Read until \r\n\r\n
            while !header_ended && llm_stream.read(&mut buf).await? > 0 {
                ollama_headers.push(buf[0]);
                if ollama_headers.ends_with(b"\r\n\r\n") {
                    header_ended = true;
                }
            }

            // 2. Only inject CORS, don't modify other headers
            let mut headers = String::from_utf8(ollama_headers)?;
            headers = headers.replace(
                "HTTP/1.1 200 OK\r\n",
                "HTTP/1.1 200 OK\r\n\
            Access-Control-Allow-Origin: *\r\n",
            );

            // 3. Send modified headers
            client_stream.write_all(headers.as_bytes()).await?;

            // 4. Stream raw NDJSON without chunked encoding
            let mut chunk = [0u8; 4096];
            let mut buffer = Vec::new();
            let mut full_collected = String::new();
            loop {
                let bytes_read = match llm_stream.read(&mut chunk).await {
                    Ok(0) => {
                        println!("Ollama connection closed");
                        break;
                    }
                    Ok(n) => {
                        println!("Forwarding {} bytes", n);
                        n
                    }
                    Err(e) => {
                        eprintln!("Read error: {}", e);
                        break;
                    }
                };
                if let Err(e) = client_stream.write_all(&chunk[..bytes_read]).await {
                    eprintln!("Write error: {}", e);
                    break;
                }
                // 3. Check for termination marker
                buffer.extend_from_slice(&chunk[..bytes_read]);
                if buffer.windows(2).any(|w| w == b"}\n") {
                    if let Ok(text) = String::from_utf8(buffer.clone()) {
                        println!("text : {}", text);
                        if text.contains("\"done\":true") {
                            println!("Detected stream end");
                            break;
                        }
                    }
                    buffer.clear();
                }

                // 4. Collected
                let collected = Self::collect_responses(&chunk)
                    .await
                    .expect("Error Collect");
                full_collected += &collected;
            }
            client_stream.write_all(b"0\r\n\r\n").await?;
            llm_stream.shutdown().await?;
            println!("end, reponse full:");
            println!("{}", full_collected);
        }
        // 4. Clean shutdown
        client_stream.shutdown().await?;
        println!("Connection closed properly");
        Ok(())
    }

    async fn collect_responses(chunk: &[u8; 4096]) -> Result<String, Box<dyn Error>> {
        let mut response_buffer = String::new();
        let mut json_buffer = Vec::new();
        let mut in_response = false;
        let mut escape_next = false;
        let mut response_bytes = Vec::new();
        for &byte in chunk {
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
                    break;
                }
            }
        }
        Ok(response_buffer)
    }
}
