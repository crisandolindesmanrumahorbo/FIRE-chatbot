use serde_json::json;

use crate::{cfg::get_config, http_client::{HttpClient, HttpMethod}, notification::{model::{PushSubscription, PushSubscriptionKeys}, svc::Notification}, 
    telegram::model::{GetUpdatesResp, LlamaRequest, LlamaResponse, OrderForm, OrderRequest, TeleMessage}};

pub struct Telegram {}

impl Telegram {
    pub async fn handle_tele_polling(mut update_id: i64) -> i64 {
        println!("Listen tele to update:\n{:?}", update_id);
        let url = format!(
            "{}/bot{}/getUpdates?offset={}",
            get_config().tele_url,
            get_config().tele_token,
            update_id
        );
        let mut response = HttpClient::fetch::<()>(HttpMethod::GET, url, None).await;
        if response.status != 200 {
            return update_id;
        }
        let body = match response.body {
            Some(body) => body,
            None => {
                println!("None body");
                return update_id;
            }
        };
        let chat: GetUpdatesResp = serde_json::from_str(&body).expect("error deserialize body");
        if chat.result.len() == 0 {
            return update_id;
        }
        update_id = chat.result[0].update_id + 1;

        let text = format!("Echo {}", chat.result[chat.result.len() - 1].message.text);
        let llama_url = String::from("http://127.0.0.1:11434/api/generate");
        let llama_body = LlamaRequest {
            model: String::from("gemma3:1b"),
            prompt: Self::build_prompt(&text),
            stream: false,
        };
        response =
            HttpClient::fetch::<LlamaRequest>(HttpMethod::POST, llama_url, Some(llama_body)).await;
        let llama_res_body = match response.body {
            Some(body) => body,
            None => {
                println!("None body");
                return update_id;
            }
        };

        let llama_response: LlamaResponse =
            serde_json::from_str(&llama_res_body).expect("error deserialize body");
        println!("Response llama:\n{:?}", llama_response.response);

        let order: OrderForm = serde_json::from_str(&llama_response.response).expect("order error");
        let order_req = OrderRequest {
            expiry: String::from("GTC"),
            user_id: 10,
            price: order.price,
            lot: order.lot,
            symbol: order.symbol.clone(),
            side: 'B',
        };
        let body = format!("{} order succeed", &order_req.symbol);
        let order_url = String::from("http://127.0.0.1:7878/order");
        response =
            HttpClient::fetch::<OrderRequest>(HttpMethod::POST, order_url, Some(order_req)).await;

        // TODO send order to service order
        // handle auth
        // packaging http client
        let body = match response.status {
            200 => {
                let push_subricption = PushSubscription { 
                    endpoint: "https://fcm.googleapis.com/fcm/send/czgxp_p8tFg:APA91bHsEh0GH49x3L6_aNQWz1pKmHfgKMCL11Kf6Wsf49tT1jbM64Ltr6_4toku6PQDdA-1O2w8yO2PH6NyVkKwy_S8o7vVUmTwIr-DTtNBp5e0ufDlNbqOXQ9wD2WFsXCM1FiOesue".to_string(), 
                    expiration_time: None, 
                    keys: PushSubscriptionKeys {
                        p256dh: "BFFGrinjE3VIjgQD3XMX-h4dh8WWCK2ifCWin9ENcwCPff_fEEYFOUTP3aIiUjaaGHYVULoH2UM7qPI0uCU_nR0".to_string(),
                        auth: "gN0P_D1siTLc1nJnRtBV8Q".to_string()
                    } 
                };
                let payload = json!({
                    "title": "",
                    "body": body,
                }).to_string();

                if let Err(e) =
                    Notification::send_web_push(&payload, &push_subricption, get_config().vapid_private_key)
                        .await
                {
                    eprintln!("connection error {}", e);
                }

                TeleMessage {
                    chat_id: chat.result[0].message.chat.id,
                    text: format!("Succeed Buy \n {:?}", order),
                }
            }
            _ => TeleMessage {
                chat_id: chat.result[0].message.chat.id,
                text: format!("Failed Buy \n {:?}", order),
            },
        };

        let url = format!(
            "{}/bot{}/sendMessage",
            get_config().tele_url,
            get_config().tele_token
        );
        let response = HttpClient::fetch::<TeleMessage>(HttpMethod::POST, url, Some(body)).await;
        println!("Response latest message:\n{:?}", response);
        update_id
    }

    fn build_prompt(user_input: &str) -> String {
        format!(
            "You are a strict trading order formatter.\n\
        Given a user's message about a trade order, extract and return ONLY a valid JSON object like this:\n\
        {{\"symbol\": \"SYMBOL\", \"price\": NUMBER, \"lot\": NUMBER, \"side\": \"B\" or \"S\"}}\n\n\
        Rules:\n\
        - Output ONLY the JSON object. No explanations or formatting like backticks.\n\
        - Keys must be: symbol, price, lot, side.\n\
        - Convert symbol to uppercase.\n\
        - Only allow known trading symbols like: XBTUSD, ETHUSD, SOLUSD, DOGEUSD, SUIUSD and etc .\n\
        - Convert human-like numbers (e.g. \"ten\", \"1 million\", \"10K\") into pure integers.\n\
        - Use \"B\" for buy and \"S\" for sell.\n\
        - Only return one order, even if the message mentions several.\n\
        - If key information is missing, return {{}}.\n\n\
        User: {}\n\
        Output:",
            user_input
        )
    }

}
