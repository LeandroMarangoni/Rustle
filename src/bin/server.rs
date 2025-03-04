use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

async fn start_server() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    println!("WebSocket server running on port 8080...");

    loop {
        let (stream, _) = listener.accept().await?;
        let ws_stream = accept_async(stream).await?;

        tokio::spawn(handle_client(ws_stream));
    }
}

async fn handle_client(mut ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>) {
    println!("New connected client!");

    while let Some(message) = ws_stream.next().await {
        match message {
            Ok(Message::Text(text)) => {
                println!("Received from client: {}", text);
                if let Err(e) = ws_stream
                    .send(Message::Text("Welcome to the node!".to_string()))
                    .await
                {
                    println!("Error sending message to client: {}", e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                println!("Client requested connection closure.");
                break;
            }
            Ok(_) => println!("Received an unexpected message type."),
            Err(e) => {
                println!("WebSocket Error: {}", e);
                break;
            }
        }
    }

    println!("Closing connection with client...");
    let _ = ws_stream.send(Message::Close(None)).await;
    println!("Client disconnected.");
}

#[tokio::main]
async fn main() {
    if let Err(e) = start_server().await {
        println!("Server error: {}", e);
    }
}
