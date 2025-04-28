use request_http_parser::parser::Request;
use tokio::io::AsyncWrite;

use crate::{
    llama_client::LlamaClient,
    server::{BAD_REQUEST, OK_RESPONSE},
};

pub struct ChatbotService {}

impl ChatbotService {
    pub async fn chatbot_streaming(
        request: &Request,
        writer: impl AsyncWrite + Unpin,
    ) -> (String, String) {
        let json_body = match &request.body {
            Some(body) => body,
            None => return (BAD_REQUEST.to_string(), "body".to_string()),
        };
        let mut llm_client = LlamaClient::new().await;
        llm_client.send(json_body).await;
        let res_msg = llm_client.stream_response(writer).await;

        // todo save response as history
        println!("full response llm \n: {}", res_msg);
        (OK_RESPONSE.to_string(), "".to_string())
    }
}
