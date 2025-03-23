use crate::blockchain::transaction::Transaction;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub previous_hash: String,
    pub hash: String,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(index: u64, previous_hash: String, transactions: Vec<Transaction>) -> Self {
        let timestamp: i64 = Utc::now().timestamp();
        let hash = Block::calculate_hash(index, timestamp, &previous_hash, &transactions);

        Block {
            index,
            timestamp,
            previous_hash,
            hash,
            transactions,
        }
    }

    pub fn calculate_hash(
        index: u64,
        timestamp: i64,
        previous_hash: &str,
        transactions: &Vec<Transaction>,
    ) -> String {
        let input: String = format!("{}{}{}{:?}", index, timestamp, previous_hash, transactions);
        let mut hasher = Sha256::new();
        hasher.update(input);
        format!("{:x}", hasher.finalize())
    }
}
