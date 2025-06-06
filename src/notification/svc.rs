use super::model::PushSubscription;
use crate::{
    cfg::get_config,
    server::{BAD_REQUEST, OK_RESPONSE},
};

use anyhow::Result;
use request_http_parser::parser::Request as HttpRequest;
use ring::{
    rand::SystemRandom,
    signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair},
};
use serde::Serialize;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use reqwest::{Client, Version};
use serde_json::json;
use std::error::Error;

#[derive(Serialize)]
pub struct PushMessage {
    pub message: String,
}

pub struct Notification {}

const PADDING_LENGTH: usize = 86; // Adjust to meet minimum size requirement

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
        // if let Err(err) = Self::send_push(
        //     &push_subricption,
        //     payload,
        //     "crisandolindummy@gmail.com",
        //     vapid_public,
        //     vapid_private,
        // )
        // .await
        if let Err(err) =
            Self::send_web_push(&push_subricption, get_config().vapid_private_key).await
        {
            eprintln!("Push error: {err}");
            return (BAD_REQUEST.to_string(), "".to_string());
        }
        (OK_RESPONSE.to_string(), "".to_string())
    }

    pub async fn send_push(
        sub: &PushSubscription,
        payload: PushMessage,
        vapid_email: &str,
        vapid_public_key: &str,
        vapid_private_key: &str,
    ) -> Result<(), Box<dyn Error>> {
        let jwt_header = json!({ "alg": "ES256", "typ": "JWT" });
        let audience = Self::get_audience(&sub.endpoint)?;
        println!("audience {}", audience);
        let jwt_claims = json!({
            "aud":audience,
            "exp": (Utc::now() + chrono::Duration::minutes(5)).timestamp(),
            "sub": format!("mailto:{}", vapid_email),
        });

        let encoded_header = URL_SAFE_NO_PAD.encode(serde_json::to_string(&jwt_header)?);
        let encoded_claims = URL_SAFE_NO_PAD.encode(serde_json::to_string(&jwt_claims)?);
        let signed_data = format!("{}.{}", encoded_header, encoded_claims);
        let signature = Self::sign_with_vapid(&signed_data, vapid_private_key, vapid_public_key)?;
        let jwt = format!("{}.{}", signed_data, URL_SAFE_NO_PAD.encode(signature));

        let client = Client::builder()
            .http2_prior_knowledge() // Use HTTP/2 directly
            .build()
            .expect("error client");
        println!("headers {}", &jwt);
        println!("endpoint {}", &sub.endpoint);

        // 1. Prepare the payload with padding to meet minimum size
        let payload = json!({
            "title": "Hello from Rust!",
            "body": "This push notification was sent from a Rust backend",
            "icon": "/icon.png"
        });

        let mut payload_str = payload.to_string();
        // Add padding if needed
        if payload_str.len() < PADDING_LENGTH {
            payload_str.push_str(&" ".repeat(PADDING_LENGTH - payload_str.len()));
        }

        let res = client
            .post(&sub.endpoint)
            .version(Version::HTTP_2)
            .header("TTL", "60")
            .header("Authorization", format!("WebPush {}", jwt))
            .header("Crypto-Key", format!("p256ecdsa={}", vapid_public_key)) // ← ADD THIS
            .header("Content-Encoding", "aes128gcm")
            .body(Vec::new())
            .send()
            .await
            .expect("error sending push");

        println!("Push sent. Status: {}", res.status());
        Ok(())
    }

    pub async fn send_web_push(
        sub: &PushSubscription,
        vapid_priv_key_b64: &str, // Base64 URL-safe (no padding) VAPID private key
    ) -> Result<()> {
        use web_push::*;
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
        let payload = json!({
            "title": "Hello from Rust!",
            "body": "This is first message",
        })
        .to_string();

        let mut msg = WebPushMessageBuilder::new(&sub);
        msg.set_payload(ContentEncoding::Aes128Gcm, payload.as_bytes());
        msg.set_vapid_signature(vapid_signature);
        msg.set_ttl(60);

        let client = IsahcWebPushClient::new()?;

        client.send(msg.build()?).await?;

        Ok(())
    }
    fn get_audience(endpoint: &str) -> Result<String, Box<dyn Error>> {
        let url = url::Url::parse(endpoint)?;
        Ok(format!("{}://{}", url.scheme(), url.host_str().unwrap()))
    }

    fn sign_with_vapid(
        data: &str,
        base64_private_key: &str,
        base64_public_key: &str,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let rng = SystemRandom::new();

        let priv_bytes = URL_SAFE_NO_PAD.decode(base64_private_key)?;
        let mut pub_bytes = URL_SAFE_NO_PAD.decode(base64_public_key)?;

        if pub_bytes.len() == 64 {
            pub_bytes.insert(0, 0x04);
        }

        assert_eq!(pub_bytes.len(), 65);
        assert_eq!(pub_bytes[0], 0x04);

        let key_pair = EcdsaKeyPair::from_private_key_and_public_key(
            &ECDSA_P256_SHA256_FIXED_SIGNING,
            &priv_bytes,
            &pub_bytes,
            &rng,
        )
        .expect("key_pair");

        let sig = key_pair.sign(&rng, data.as_bytes()).expect("Sig");
        Ok(sig.as_ref().to_vec())
    }
}
