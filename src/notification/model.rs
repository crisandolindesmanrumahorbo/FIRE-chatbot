use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PushSubscriptionKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Deserialize)]
pub struct PushSubscription {
    pub endpoint: String,
    pub expiration_time: Option<u64>, // nullable in JSON
    pub keys: PushSubscriptionKeys,
}
