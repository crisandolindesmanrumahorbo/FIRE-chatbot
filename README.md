## How integrate Web Push Notification
![alt text](image-4.png)

## How Chatbot integration works
![alt text](image-3.png)

## How Telegram integration works
![alt text](image-5.png)

## How to run
```
cargo run
```

## Setup
- First get ollama model
- Run docker ollama
```
docker run -d -v ollama:/root/.ollama -p 11434:11434 --name ollama ollama/ollama
```

generate vapid private and public key on https://tools.reactpwa.com/vapid