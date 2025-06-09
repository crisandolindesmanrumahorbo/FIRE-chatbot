use crate::chatbot_stream::svc::ChatbotStreamService;
use crate::notification::svc::Notification;
use crate::telegram::svc::Telegram;

use anyhow::{Context, Result};
use request_http_parser::parser::{Method, Request};
use std::error::Error;
use tokio::io::AsyncWrite;
use tokio::time::{Duration, sleep};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::oneshot::Receiver,
};

pub const BAD_REQUEST: &str = "HTTP/1.1 400 Bad Request\r\n\r\n";
pub const NOT_FOUND: &str = "HTTP/1.1 404 Not Found\r\n\r\n";
pub const OPTIONS_CORS: &str = "HTTP/1.1 204 No Content\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Access-Control-Allow-Methods: POST, GET, OPTIONS\r\n\
            Access-Control-Allow-Headers: Content-Type\r\n\
            Access-Control-Max-Age: 86400\r\n\
            \r\n";
pub const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Access-Control-Allow-Methods: POST, GET, OPTIONS\r\n\
            Access-Control-Allow-Headers: Content-Type\r\n\
            Access-Control-Max-Age: 86400\r\n\
            Content-Type: application/json\r\n\
            \r\n";

pub struct Server {}
impl Server {
    pub async fn start(mut shutdown_rx: Receiver<()>) -> Result<()> {
        let listener = TcpListener::bind("localhost:8080")
            .await
            .expect("bind port error");
        println!("Server running on http://localhost:8080");
        let mut update_id = 616646863;

        loop {
            tokio::select! {
                conn = listener.accept() => {
                    let (mut stream, _) = conn?;
                    tokio::spawn(async move {
                        let (reader, writer) = stream.split();
                        if let Err(e) = Self::handle_client(reader, writer).await {
                            eprintln!("connection error {}", e);
                        }
                    });
                }
                _ = &mut shutdown_rx => {
                    println!("Shutting down server...");
                    break;
                }
                _ = sleep(Duration::from_secs(2)) => {
                    update_id = Telegram::handle_tele_polling(update_id).await;
                }
            }
        }

        Ok(())
    }

    pub async fn handle_client<Reader, Writer>(
        mut reader: Reader,
        mut writer: Writer,
    ) -> Result<(), Box<dyn Error>>
    where
        Reader: AsyncRead + Unpin,
        Writer: AsyncWrite + Unpin,
    {
        let mut buffer = [0; 4048];
        let size = reader
            .read(&mut buffer)
            .await
            .context("Failed to read stream")?;
        if size >= 4048 {
            println!("too large");
            let _ = writer
                .write_all(format!("{}{}", BAD_REQUEST, "Requets too large").as_bytes())
                .await
                .context("Failed to write");

            let _ = writer.flush().await.context("Failed to flush");

            return Ok(());
        }
        let request = String::from_utf8_lossy(&buffer[..size]);
        let request = match Request::new(&request) {
            Ok(req) => req,
            Err(e) => {
                println!("{}", e);
                let _ = writer
                    .write_all(format!("{}{}", BAD_REQUEST, e).as_bytes())
                    .await
                    .context("Failed to write");

                let _ = writer.flush().await.context("Failed to flush");
                return Ok(());
            }
        };

        // Router
        let (content, status) = match (&request.method, request.path.as_str()) {
            (Method::OPTIONS, _) => ("".to_string(), OPTIONS_CORS.to_string()),
            (Method::POST, "/chatbot") => {
                ChatbotStreamService::chatbot_streaming(&request, &mut writer).await
            }
            (Method::POST, "/register-subscription") => Notification::register_subs(&request).await,
            (Method::POST, "/push-notification") => Notification::push_notification(&request).await,
            _ => (NOT_FOUND.to_string(), "404 Not Found".to_string()),
        };

        let _ = writer
            .write_all(format!("{}{}", status, content).as_bytes())
            .await
            .context("Failed to write");

        Ok(())
    }
}
