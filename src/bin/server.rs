use futures_util::{SinkExt, StreamExt};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, RwLock};
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::protocol::Message};

use rust_blockchain::peer::Peer;
use rust_blockchain::storage::{load_peers, save_peers};

use rust_blockchain::blockchain::block::Block;
use rust_blockchain::blockchain::blockchain::Blockchain;

async fn start_server(
    peers: Arc<RwLock<HashSet<Peer>>>,
    blockchain: Arc<RwLock<Blockchain>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("WebSocket server running on port 8080...");

    loop {
        let (stream, addr) = listener.accept().await?;
        let ws_stream = Arc::new(Mutex::new(accept_async(stream).await?));

        let peer = Peer { address: addr };
        add_peer(peers.clone(), peer).await;
        save_peers(&peers).await;

        let peers_message = get_peers_message(peers.clone()).await?;
        send_message(&ws_stream, peers_message).await?;

        let block_message = get_block_message(blockchain.clone()).await?;
        send_message(&ws_stream, block_message).await?;

        tokio::spawn(handle_client(ws_stream.clone(), blockchain.clone()));
        tokio::spawn(heartbeat(ws_stream.clone()));
    }
}

async fn add_peer(peers: Arc<RwLock<HashSet<Peer>>>, peer: Peer) {
    let mut peers_guard = peers.write().await;
    peers_guard.insert(peer);
}

async fn get_peers_message(
    peers: Arc<RwLock<HashSet<Peer>>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let peers_list = {
        let peers_guard = peers.read().await;
        serde_json::to_string(&*peers_guard)?
    };
    Ok(peers_list)
}

async fn get_block_message(
    blockchain: Arc<RwLock<Blockchain>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let block = {
        let blockchain_guard = blockchain.read().await;
        serde_json::to_string(&blockchain_guard.chain[0])?
    };
    Ok(block)
}

async fn send_message(
    ws_stream: &Arc<Mutex<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>>>,
    message: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut ws_stream = ws_stream.lock().await;
    ws_stream.send(Message::Text(message)).await?;
    Ok(())
}

async fn heartbeat(ws_stream: Arc<Mutex<WebSocketStream<tokio::net::TcpStream>>>) {
    loop {
        let mut ws_stream = ws_stream.lock().await;

        if let Err(e) = ws_stream.send(Message::Ping(Vec::new())).await {
            println!("Error sending heartbeat: {}", e);
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
}

async fn handle_client(
    ws_stream: Arc<Mutex<WebSocketStream<tokio::net::TcpStream>>>,
    blockchain: Arc<RwLock<Blockchain>>,
) {
    let mut ws_stream = ws_stream.lock().await;

    while let Some(message) = ws_stream.next().await {
        match message {
            Ok(Message::Text(block_json)) => match serde_json::from_str::<Block>(&block_json) {
                Ok(block) => {
                    let blockchain_read = blockchain.read().await;

                    if blockchain_read.is_valid_block(&block) {
                        let mut blockchain_write = blockchain.write().await;
                        blockchain_write.propagate_block(&block).await;
                        blockchain_write.add_block(block.transactions);
                        println!("Received and added block");
                    } else {
                        println!("Invalid block received: {:?}", block);
                    }
                }
                Err(e) => {
                    println!("Error deserializing block: {}", e);
                }
            },
            _ => {}
        }
    }

    println!("Client disconnected.");
}

#[tokio::main]
async fn main() {
    let peers = Arc::new(RwLock::new(load_peers().await));
    let blockchain = Arc::new(RwLock::new(Blockchain::new()));
    if let Err(e) = start_server(peers, blockchain).await {
        println!("Server error: {}", e);
    }
}
