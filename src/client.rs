use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct Client {}

impl Client {
    pub async fn perform(is_stream: bool, json_body: &str, mut client_stream: TcpStream) {
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

            let mut headers = [0u8; 1024];
            let header_len = llm_stream.read(&mut headers).await.expect("header error");
            client_stream
                .write_all(&headers[..header_len])
                .await
                .expect("Error write once");

            // Process streaming response
            let mut chunk = [0; 1024];
            loop {
                let bytes_read = match llm_stream.read(&mut chunk).await {
                    Ok(0) => {
                        println!("closed llm connection");
                        break;
                    } // Connection closed by Ollama
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Read error: {}", e);
                        break;
                    }
                };

                client_stream
                    .write_all(&chunk[..bytes_read])
                    .await
                    .expect("Eerror chunk write");

                let _ = client_stream.flush().await;
            }
        }
    }
}
