use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use tokio::time::{Duration, sleep};
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
        ws_stream.send(Message::Text("heartbeat".into())).await?;
        println!("Sent heartbeat");

        tokio::select! {
            Some(Ok(Message::Text(text))) = ws_stream.next() => {
                println!("Received message: {}", text);
            }
            _ = sleep(Duration::from_secs(5)) => {}
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
