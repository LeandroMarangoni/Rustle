mod block;
mod blockchain;
mod transaction;

use blockchain::Blockchain;
use transaction::Transaction;

fn main() {
    let mut blockchain = Blockchain::new();

    let tx1 = Transaction::new("Alice".to_string(), "Bob".to_string(), 10.0);
    let tx2 = Transaction::new("Bob".to_string(), "Charlie".to_string(), 5.0);

    blockchain.add_block(vec![tx1, tx2]);

    if blockchain.is_valid() {
        println!("✅ Valid blockchain!");
    } else {
        println!("❌ Invalid blockchain!");
    }

    blockchain.chain[1].transactions[0].amount = 1000.0;

    if blockchain.is_valid() {
        println!("✅ Blockchain still valid!");
    } else {
        println!("❌ Alert! Blockchain has been tampered with!");
    }
}
