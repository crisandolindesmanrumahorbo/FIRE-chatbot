use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::Serialize;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use url::Url;

pub enum HttpMethod {
    GET,
    POST,
}

pub struct HttpClient {}

impl HttpClient {
    pub async fn fetch<T: Serialize>(method: HttpMethod, url: String, body: Option<T>) -> Response {
        let parsed = Url::parse(&url).expect("Invalid Url");

        let _scheme = parsed.scheme();
        let host = parsed.host_str().expect("error url host");
        let port = parsed.port_or_known_default().expect("error url port");
        let path = parsed.path();
        let full_path = match parsed.query() {
            Some(query) => format!("{}?{}", path, query),
            None => path.to_string(),
        };
        let conn = TcpStream::connect((host, port))
            .await
            .expect(format!("Failed to handshake to {}", url).as_str());
        let tls_connector = native_tls::TlsConnector::new().expect("error init tls");
        let connector = tokio_native_tls::TlsConnector::from(tls_connector);

        let mut stream = connector
            .connect(host, conn)
            .await
            .expect("TLS Handshake failed");

        let req = match method {
            HttpMethod::GET => format!(
                "GET {} HTTP/1.1\r\n\
                    Host: {}\r\n\
                    Connection: close\r\n\
                    \r\n",
                full_path, host
            ),
            HttpMethod::POST => {
                let res = serde_json::to_string(&body).expect("error serialize");
                let request = format!(
                    "POST /bot7994038141:AAFIcLqsTY_xI-eAsv32l1-JEAVTx9Y8-Ks/sendMessage HTTP/1.1\r\n\
                    Host: {}\r\n\
                    Content-Type: application/json\r\n\
                    Content-Length: {}\r\n\
                    Connection: close\r\n\
                    \r\n\
                    {}",
                    host,
                    res.len(),
                    res
                );
                request
            }
        };
        stream
            .write_all(req.as_bytes())
            .await
            .expect(format!("failed to write to {}", url).as_str());
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

        response
    }
}

#[derive(Debug)]
pub struct Response {
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
