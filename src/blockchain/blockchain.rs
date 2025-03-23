use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::blockchain::block::Block;
use crate::blockchain::transaction::Transaction;
use crate::peer::Peer;

use futures_util::sink::SinkExt;
use tokio::sync::RwLock as AsyncRwLock;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Clone)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub peers: Arc<AsyncRwLock<HashSet<Peer>>>,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis_transactions = vec![Transaction::new(
            "System".to_string(),
            "Genesis".to_string(),
            0.0,
        )];
        let genesis_block = Block::new(0, "0".to_string(), genesis_transactions);

        Blockchain {
            chain: vec![genesis_block],
            peers: Arc::new(AsyncRwLock::new(HashSet::new())),
        }
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) {
        let previous_block: &Block = self.chain.last().unwrap();
        let new_block = Block::new(
            previous_block.index + 1,
            previous_block.hash.clone(),
            transactions,
        );
        self.chain.push(new_block);
    }

    pub async fn propagate_block(&self, block: &Block) {
        let peers = self.peers.read().await;
        for peer in peers.iter() {
            let address = peer.address.clone();
            tokio::spawn(send_block_to_peer(address, block.clone()));
        }
    }

    pub fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            if current.previous_hash != previous.hash {
                return false;
            }

            let recalculated_hash = Block::calculate_hash(
                current.index,
                current.timestamp,
                &current.previous_hash,
                &current.transactions,
            );

            if current.hash != recalculated_hash {
                return false;
            }
        }
        true
    }

    pub fn is_valid_block(&self, new_block: &Block) -> bool {
        let last_block = self.chain.last().unwrap();

        if new_block.index != last_block.index + 1 {
            return false;
        }

        if new_block.previous_hash != last_block.hash {
            return false;
        }

        if new_block.hash
            != Block::calculate_hash(
                new_block.index,
                new_block.timestamp,
                &new_block.previous_hash,
                &new_block.transactions,
            )
        {
            return false;
        }

        true
    }
}

async fn send_block_to_peer(peer_address: SocketAddr, block: Block) {
    let url = format!("ws://{}", peer_address);

    if let Ok((mut ws_stream, _)) = connect_async(&url).await {
        match serde_json::to_string(&block) {
            Ok(block_message) => {
                if let Err(e) = ws_stream.send(Message::Text(block_message)).await {
                    println!("Error sending block to peer: {}", e);
                }
            }
            Err(e) => {
                println!("Error serializing block: {}", e);
            }
        }
    } else {
        println!("Error connecting to peer at {}", url);
    }
}
