// src/websocket.rs - WebSocket Server Implementation

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde_json::Value as JsonValue;

pub type ClientId = usize;
pub type Clients = Arc<Mutex<HashMap<ClientId, tokio::sync::mpsc::UnboundedSender<Message>>>>;

pub struct WebSocketServer {
    port: u16,
    clients: Clients,
    next_client_id: Arc<Mutex<ClientId>>,
}

impl WebSocketServer {
    pub fn new(port: u16) -> Self {
        WebSocketServer {
            port,
            clients: Arc::new(Mutex::new(HashMap::new())),
            next_client_id: Arc::new(Mutex::new(1)),
        }
    }

    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        
        println!("🔌 WebSocket server listening on: ws://{}", addr);

        while let Ok((stream, peer_addr)) = listener.accept().await {
            let clients = Arc::clone(&self.clients);
            let next_id = Arc::clone(&self.next_client_id);
            
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, clients, next_id, peer_addr).await {
                    eprintln!("❌ WebSocket error: {}", e);
                }
            });
        }

        Ok(())
    }
}

async fn handle_connection(
    stream: TcpStream,
    clients: Clients,
    next_id: Arc<Mutex<ClientId>>,
    peer_addr: std::net::SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    
    let client_id = {
        let mut id = next_id.lock().unwrap();
        let current = *id;
        *id += 1;
        current
    };

    println!("✅ Client {} connected from {}", client_id, peer_addr);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    {
        let mut clients_lock = clients.lock().unwrap();
        clients_lock.insert(client_id, tx);
    }

    let welcome = serde_json::json!({
        "type": "connected",
        "client_id": client_id,
        "message": "Welcome to Nova WebSocket!"
    });
    ws_sender.send(Message::Text(welcome.to_string())).await?;

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("📨 Client {} sent: {}", client_id, text);
                
                if let Ok(json) = serde_json::from_str::<JsonValue>(&text) {
                    handle_message(client_id, json, &clients).await;
                }
            }
            Ok(Message::Close(_)) => {
                println!("👋 Client {} disconnected", client_id);
                break;
            }
            Ok(Message::Ping(data)) => {
                let clients_lock = clients.lock().unwrap();
                if let Some(tx) = clients_lock.get(&client_id) {
                    let _ = tx.send(Message::Pong(data));
                }
            }
            Err(e) => {
                eprintln!("❌ Client {} error: {}", client_id, e);
                break;
            }
            _ => {}
        }
    }

    {
        let mut clients_lock = clients.lock().unwrap();
        clients_lock.remove(&client_id);
    }

    send_task.abort();
    
    Ok(())
}

async fn handle_message(client_id: ClientId, message: JsonValue, clients: &Clients) {
    if let Some(msg_type) = message.get("type").and_then(|v| v.as_str()) {
        match msg_type {
            "broadcast" => {
                if let Some(content) = message.get("content") {
                    let broadcast_msg = serde_json::json!({
                        "type": "message",
                        "from": client_id,
                        "content": content
                    });
                    
                    let clients_lock = clients.lock().unwrap();
                    let msg = Message::Text(broadcast_msg.to_string());
                    
                    for (_id, tx) in clients_lock.iter() {
                        let _ = tx.send(msg.clone());
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn start_websocket_server(port: u16) {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            let server = WebSocketServer::new(port);
            if let Err(e) = server.start().await {
                eprintln!("❌ WebSocket server error: {}", e);
            }
        });
}