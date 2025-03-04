use futures_util::{SinkExt, StreamExt};
use tokio::time::{Duration, sleep};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

async fn start_client() -> Result<(), Box<dyn std::error::Error>> {
    let url = "ws://localhost:8080";
    let (mut ws_stream, _) = connect_async(url).await?;

    println!("Connected to WebSocket server!");

    ws_stream
        .send(Message::Text("Hello server!".to_string()))
        .await?;

    if let Some(response) = ws_stream.next().await {
        match response {
            Ok(Message::Text(text)) => {
                println!("Received from server: {}", text);
            }
            Ok(_) => println!("Received an unexpected message type"),
            Err(e) => println!("Error receiving message: {}", e),
        }
    }

    println!("Waiting before closing connection...");
    sleep(Duration::from_secs(2)).await;

    ws_stream.send(Message::Close(None)).await?;
    println!("Connection closed correctly.");

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = start_client().await {
        println!("Client error: {}", e);
    }
}
