use futures_util::{SinkExt, StreamExt};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

use rust_blockchain::peer::Peer;

async fn start_server(peers: Arc<RwLock<HashSet<Peer>>>) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("WebSocket server running on port 8080...");

    loop {
        let (stream, addr) = listener.accept().await?;
        let mut ws_stream = accept_async(stream).await?;

        let peer = Peer::new(addr);
        peers.write().await.insert(peer.clone());

        let peers_list = peers.read().await.clone();
        let peers_message = serde_json::to_string(&peers_list).unwrap();
        ws_stream.send(Message::Text(peers_message)).await.unwrap();

        tokio::spawn(handle_client(ws_stream, peers.clone()));
    }
}

async fn handle_client(
    mut ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    peers: Arc<RwLock<HashSet<Peer>>>,
) {
    while let Some(message) = ws_stream.next().await {
        match message {
            Ok(Message::Text(text)) => {
                println!("Received from client: {}", text);
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
    let peers = Arc::new(RwLock::new(HashSet::new()));
    if let Err(e) = start_server(peers).await {
        println!("Server error: {}", e);
    }
}
