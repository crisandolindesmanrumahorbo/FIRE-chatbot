use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use stockbit_chatbot::{cfg::init_config, server::Server, tele::GetUpdatesResp};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    signal::unix::{SignalKind, signal},
    sync::oneshot,
    time::sleep,
};
use tokio_native_tls::TlsConnector;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    handle_tele_bot().await;
    // init_config();
    //
    // let (shutdown_tx, shutdown_rx) = oneshot::channel();
    //
    // let server_handle = tokio::spawn(async move {
    //     let _ = Server::start(shutdown_rx).await;
    // });
    //
    // gracefully_shutdown(shutdown_tx, server_handle).await;

    Ok(())
}

async fn gracefully_shutdown(
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    server_handle: tokio::task::JoinHandle<()>,
) {
    let mut signal_terminate = signal(SignalKind::terminate()).unwrap();
    let mut signal_interrupt = signal(SignalKind::interrupt()).unwrap();

    tokio::select! {
        _ = signal_terminate.recv() => {
            println!("Shutdown signal_terminate received");
        },
        _ = signal_interrupt.recv() => {
                println!("Shutdown signal_interrupt received");
            }
    }
    let _ = shutdown_tx.send(());
    let _ = server_handle.await;
    println!("shutdown completed");
}

async fn handle_tele_bot() {
    let mut update_id = 616646851;
    let domain = "api.telegram.org";
    let port = 443;

    loop {
        println!("UPDATE_ID:\n{:?}", update_id);

        // TCP + TLS connection
        let conn = TcpStream::connect((domain, port))
            .await
            .expect("Failed to connect");
        let tls_connector = native_tls::TlsConnector::new().expect("error tls connector");
        let connector = TlsConnector::from(tls_connector);

        let mut stream = connector
            .connect(domain, conn)
            .await
            .expect("TLS handshake failed");

        // HTTP GET request
        let mut request = format!(
            "GET /bot7994038141:AAFIcLqsTY_xI-eAsv32l1-JEAVTx9Y8-Ks/getUpdates?offset={} HTTP/1.1\r\n\
         Host: {}\r\n\
         Connection: close\r\n\
         \r\n",
            update_id, domain
        );

        stream
            .write_all(request.as_bytes())
            .await
            .expect("write failed");

        let mut response = Vec::new();
        let mut buf = [0u8; 1024];

        loop {
            let n = stream.read(&mut buf).await.expect("read failed");
            if n == 0 {
                break;
            }
            response.extend_from_slice(&buf[..n]);
        }

        let res = String::from_utf8_lossy(&response);
        let response = Response::new(&res).expect("error deserialize response");
        println!("Response:\n{:?}", response);
        if response.status != 200 {
            break;
        }
        let body = match response.body {
            Some(body) => body,
            None => {
                println!("None body");
                return;
            }
        };
        let chat: GetUpdatesResp = serde_json::from_str(&body).expect("error deserialize body");
        if chat.result.len() == 0 {
            continue;
        }
        println!("Chat Response:\n{:?}", chat);
        update_id = chat.result[0].update_id + 1;
        let text = format!("Echo {}", chat.result[0].message.text);
        let body = TeleMessage {
            chat_id: chat.result[0].message.chat.id,
            text,
        };
        let res = serde_json::to_string(&body).expect("error serialize");
        request = format!(
            "POST /bot7994038141:AAFIcLqsTY_xI-eAsv32l1-JEAVTx9Y8-Ks/sendMessage HTTP/1.1\r\n\
            Host: {}\r\n\
            Content-Type: application/json\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\
            \r\n\
            {}",
            domain,
            res.len(),
            res
        );
        let conn = TcpStream::connect((domain, port))
            .await
            .expect("Failed to connect");
        let tls_connector = native_tls::TlsConnector::new().expect("error tls connector");
        let connector = TlsConnector::from(tls_connector);

        let mut stream = connector
            .connect(domain, conn)
            .await
            .expect("TLS handshake failed");
        stream
            .write_all(request.as_bytes())
            .await
            .expect("error write");
        let mut response = Vec::new();
        let mut buf = [0u8; 1024];

        loop {
            let n = stream.read(&mut buf).await.expect("read failed");
            if n == 0 {
                break;
            }
            response.extend_from_slice(&buf[..n]);
        }

        let res = String::from_utf8_lossy(&response);
        let response = Response::new(&res).expect("error deserialize response");
        println!("Response after post:\n{:?}", response);

        sleep(std::time::Duration::from_millis(2000)).await;
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TeleMessage {
    pub chat_id: i64,
    pub text: String,
}

#[derive(Debug)]
struct Response {
    pub status: i32,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Option<String>,
}

impl Response {
    pub fn new(request: &str) -> Result<Self> {
        let mut parts = request.split("\r\n\r\n");
        let head = parts.next().context("Headline Error")?;
        // Body
        let body = parts
            .next()
            .map(|b| b.split("\r\n\r\n").last().unwrap_or_default().to_string());

        // Method and path
        let mut head_line = head.lines();
        let first: &str = head_line.next().context("Empty Request")?;
        let mut request_parts: std::str::SplitWhitespace<'_> = first.split_whitespace();
        let _http = request_parts.next().context("Missing Http")?;
        let status = request_parts.next().context("No Status Code")?;

        // Headers
        let mut headers = HashMap::new();
        for line in head_line {
            if let Some((k, v)) = line.split_once(":") {
                headers.insert(k.trim().to_lowercase(), v.trim().to_string());
            }
        }
        Ok(Response {
            status: status.parse::<i32>()?,
            headers,
            body,
        })
    }
}
