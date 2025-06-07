use super::model::{PushSubscription, PushSubscriptionKeys};
use crate::{
    cfg::get_config,
    server::{BAD_REQUEST, OK_RESPONSE},
};
use anyhow::Result;
use request_http_parser::parser::Request as HttpRequest;
use serde::Serialize;
use serde_json::json;
use web_push::*;

#[derive(Serialize)]
pub struct PushMessage {
    pub title: String,
    pub body: String,
}

pub struct Notification {}

impl Notification {
    pub async fn register_subs(request: &HttpRequest) -> (String, String) {
        let body = match &request.body {
            Some(body) => body,
            None => return (BAD_REQUEST.to_string(), "".to_string()),
        };
        let push_subricption = match serde_json::from_str::<PushSubscription>(body) {
            Ok(push_subricption) => push_subricption,
            Err(_) => return (BAD_REQUEST.to_string(), "".to_string()),
        };
        let payload = json!({
            "title": "Succeed",
            "body": "This is first message",
        }).to_string();
        if let Err(err) =
            Self::send_web_push(&payload, &push_subricption, get_config().vapid_private_key).await
        {
            eprintln!("Push error: {err}");
            return (BAD_REQUEST.to_string(), "".to_string());
        }
        (OK_RESPONSE.to_string(), "".to_string())
    }

    pub async fn push_notification(request: &HttpRequest) -> (String, String) {
        let body = match &request.body {
            Some(body) => body,
            None => return (BAD_REQUEST.to_string(), "".to_string()),
        };
        let push_subricption = PushSubscription { 
            endpoint: "https://fcm.googleapis.com/fcm/send/czgxp_p8tFg:APA91bHsEh0GH49x3L6_aNQWz1pKmHfgKMCL11Kf6Wsf49tT1jbM64Ltr6_4toku6PQDdA-1O2w8yO2PH6NyVkKwy_S8o7vVUmTwIr-DTtNBp5e0ufDlNbqOXQ9wD2WFsXCM1FiOesue".to_string(), 
            expiration_time: None, 
            keys: PushSubscriptionKeys {
                p256dh: "BFFGrinjE3VIjgQD3XMX-h4dh8WWCK2ifCWin9ENcwCPff_fEEYFOUTP3aIiUjaaGHYVULoH2UM7qPI0uCU_nR0".to_string(),
                auth: "gN0P_D1siTLc1nJnRtBV8Q".to_string()
            },
        };

        if let Err(err) =
            Self::send_web_push(&body, &push_subricption, get_config().vapid_private_key).await
        {
            eprintln!("Push error: {err}");
            return (BAD_REQUEST.to_string(), "".to_string());
        }

        (OK_RESPONSE.to_string(), "".to_string())
    }

    pub async fn send_web_push(
        payload: &str,
        sub: &PushSubscription,
        vapid_priv_key_b64: &str, 
    ) -> Result<()> {
        let sub = SubscriptionInfo {
            endpoint: sub.endpoint.clone(),
            keys: SubscriptionKeys {
                p256dh: sub.keys.p256dh.clone(),
                auth: sub.keys.auth.clone(),
            },
        };
        let vapid_signature = VapidSignatureBuilder::from_base64(
            vapid_priv_key_b64,
            &sub, // subscription info goes here
        )?
        .build()?;
        let mut msg = WebPushMessageBuilder::new(&sub);
        msg.set_payload(ContentEncoding::Aes128Gcm, payload.as_bytes());
        msg.set_vapid_signature(vapid_signature);
        msg.set_ttl(60);

        let client = IsahcWebPushClient::new()?;

        client.send(msg.build()?).await?;

        Ok(())
    }
}
