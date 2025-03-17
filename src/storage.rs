use crate::peer::Peer;
use serde_json;
use std::collections::HashSet;
use std::sync::Arc;
use std::{fs, io::Write};
use tokio::sync::RwLock;

const PEERS_FILE: &str = "peers.json";

pub async fn save_peers(peers: &Arc<RwLock<HashSet<Peer>>>) {
    let peers_guard = peers.read().await;
    let peers_vec: Vec<Peer> = peers_guard.iter().cloned().collect();

    if let Ok(json) = serde_json::to_string_pretty(&peers_vec) {
        if let Ok(mut file) = fs::File::create(PEERS_FILE) {
            let _ = file.write_all(json.as_bytes());
        }
    }
}

pub async fn load_peers() -> HashSet<Peer> {
    if let Ok(data) = fs::read_to_string(PEERS_FILE) {
        if let Ok(peers) = serde_json::from_str::<Vec<Peer>>(&data) {
            return peers.into_iter().collect();
        }
    }
    HashSet::new()
}
