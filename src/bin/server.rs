use futures_util::{SinkExt, StreamExt};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{Duration, sleep};
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::protocol::Message};

use rust_blockchain::peer::Peer;
use rust_blockchain::storage::{load_peers, save_peers};

async fn start_server(peers: Arc<RwLock<HashSet<Peer>>>) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("WebSocket server running on port 8080...");

    loop {
        let (stream, addr) = listener.accept().await?;
        let mut ws_stream = accept_async(stream).await?;

        let peer = Peer { address: addr };

        let peers = Arc::clone(&peers);
        peers.write().await.insert(peer.clone());

        save_peers(&peers).await;

        let peers_list = peers.read().await.clone();
        let peers_message = serde_json::to_string(&peers_list).unwrap();
        ws_stream.send(Message::Text(peers_message)).await.unwrap();

        let ws_stream = Arc::new(Mutex::new(ws_stream));
        tokio::spawn(handle_client(ws_stream.clone()));
        tokio::spawn(heartbeat(ws_stream.clone()));
    }
}

async fn heartbeat(ws_stream: Arc<Mutex<WebSocketStream<tokio::net::TcpStream>>>) {
    let heartbeat_interval = Duration::from_secs(10);

    loop {
        sleep(heartbeat_interval).await;

        let mut ws_stream = ws_stream.lock().await;

        if let Err(e) = ws_stream.send(Message::Ping(vec![])).await {
            println!("Error sending heartbeat: {}", e);
            break;
        }

        println!("Heartbeat sent");
    }
}

async fn handle_client(
    ws_stream: Arc<Mutex<WebSocketStream<tokio::net::TcpStream>>>
) {
    let mut ws_stream = ws_stream.lock().await;

    while let Some(message) = ws_stream.next().await {
        match message {
            Ok(Message::Text(text)) => {
                println!("Received from client: {}", text);
            }
            Ok(Message::Pong(_)) => {
                println!("Received Pong response from client");
            }
            Ok(_) => {}
            Err(e) => {
                println!("WebSocket error: {}", e);
                break;
            }
        }
    }

    println!("Client disconnected.");
}

#[tokio::main]
async fn main() {
    let peers = Arc::new(RwLock::new(load_peers().await));
    if let Err(e) = start_server(peers).await {
        println!("Server error: {}", e);
    }
}
