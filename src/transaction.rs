use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub timestamp: u64,
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: u64) -> Transaction {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let mut tx = Transaction {
            id: String::new(),
            sender,
            receiver,
            amount,
            timestamp,
        };

        tx.id = tx.calculate_hash();
        tx
    }

    pub fn calculate_hash(&self) -> String {
        let input = format!("{}{}{}{}", 
            self.sender, 
            self.receiver, 
            self.amount, 
            self.timestamp
        );
        
        let mut hasher = Sha256::new();
        hasher.update(input);
        hex::encode(hasher.finalize())
    }
}