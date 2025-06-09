use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderForm {
    pub symbol: String,
    pub side: char,
    pub price: u32,
    pub lot: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderRequest {
    pub symbol: String,
    pub side: char,
    pub price: u32,
    pub lot: u32,
    pub expiry: String,
    pub user_id: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LlamaResponse {
    pub response: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LlamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TeleMessage {
    pub chat_id: i64,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetUpdatesResp {
    pub ok: bool,
    pub result: Vec<TelegramUpdate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub message: Message,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub message_id: i64,
    pub from: User,
    pub chat: Chat,
    pub date: i64,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entities: Option<Vec<MessageEntity>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chat {
    pub id: i64,
    pub first_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(rename = "type")]
    pub chat_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageEntity {
    pub offset: i64,
    pub length: i64,
    #[serde(rename = "type")]
    pub entity_type: String,
}
