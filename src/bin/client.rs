use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use rust_blockchain::peer::Peer;

async fn start_client() -> Result<(), Box<dyn Error>> {
    let url = "ws://localhost:8080";
    let (mut ws_stream, _) = connect_async(url).await?;
    println!("Connected to WebSocket server!");

    if let Some(Ok(Message::Text(peers_json))) = ws_stream.next().await {
        let peers: Vec<Peer> = serde_json::from_str(&peers_json)?;
        println!("Received peers: {:?}", peers);

        for peer in peers {
            let addr = peer.address;
            connect_to_peer(addr).await?;
        }
    }

    loop {
        if let Some(Ok(message)) = ws_stream.next().await {
            match message {
                Message::Ping(_) => {
                    ws_stream.send(Message::Pong(vec![])).await?;
                    println!("Received Ping, sent Pong");
                }
                Message::Pong(_) => {
                    println!("Received Pong");
                }
                _ => {}
            }
        }
    }
}

async fn connect_to_peer(address: std::net::SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to another peer: {}", address);
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = start_client().await {
        println!("Client error: {}", e);
    }
}