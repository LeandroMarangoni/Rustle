use crate::blockchain::block::Block;
use crate::blockchain::transaction::Transaction;

pub struct Blockchain {
    pub chain: Vec<Block>,
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
        }
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) {
        let previous_block = self.chain.last().unwrap();
        let new_block = Block::new(
            previous_block.index + 1,
            previous_block.hash.clone(),
            transactions,
        );
        self.chain.push(new_block);
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
}
