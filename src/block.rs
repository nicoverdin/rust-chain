use crate::transaction::Transaction;
use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub prev_block_hash: String,
    pub hash: String,
    pub height: u64,
    pub nonce: u64,
    pub difficulty: usize,
}

impl Block {
    pub fn new(transactions: Vec<Transaction>, prev_block_hash: String, height: u64, difficulty: usize) -> Block {
        let timestamp = Utc::now().timestamp();
        let mut block = Block {
            timestamp,
            transactions,
            prev_block_hash,
            hash: String::new(),
            height,
            nonce: 0,
            difficulty,
        };
        
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let tx_data = format!("{:?}", self.transactions);

        let input = format!("{}{}{}{}{}{}", 
            self.timestamp, 
            tx_data, 
            self.prev_block_hash, 
            self.height,
            self.nonce,
            self.difficulty,
        );
        
        let mut hasher = Sha256::new();
        hasher.update(input);        
        hex::encode(hasher.finalize())
    }

    pub fn mine(&mut self) {
        let target = "0".repeat(self.difficulty);

        println!("Mining block {}...", self.height);

        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }

        println!("Block successfully mined! Nonce: {}, Hash: {}", self.nonce, self.hash);
    }

    pub fn genesis() -> Block {
        let difficulty: usize = 0; // Genesis usually has 0 difficulty for instant creation
        let genesis_tx = Transaction::new(
            "0".to_string(),
            "admin".to_string(),
            1000
        );

        let mut block = Block::new(
            vec![genesis_tx], 
            "0".to_string(),
            0,
            difficulty,
        );

        block.hash = block.calculate_hash();
        block
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_mining_difficulty() {
        let mut block = Block::genesis();
        block.difficulty = 2;
        block.mine();

        assert!(
            block.hash.starts_with("00"), 
            "Mined hash should start with 00 given difficulty 2"
        );
    }

    #[test]
    fn test_calculate_hash_consistency() {
        let block = Block::genesis();
        let hash1 = block.calculate_hash();
        let hash2 = block.calculate_hash();

        assert_eq!(hash1, hash2, "Hash calculation should be deterministic");
    }

    #[test]
    fn test_hash_changes_on_tamper() {
        let mut block = Block::genesis();
        let original_hash = block.calculate_hash();

        // 1. Change Nonce
        block.nonce += 1;
        assert_ne!(block.calculate_hash(), original_hash, "Hash must change when nonce changes");

        // Reset
        block.nonce -= 1; 
        
        // 2. Change Data (Transactions)
        let new_tx = Transaction::new("Alice".into(), "Bob".into(), 10);
        block.transactions.push(new_tx);
        assert_ne!(block.calculate_hash(), original_hash, "Hash must change when tx data changes");
    }

    #[test]
    fn test_genesis_block_structure() {
        let genesis = Block::genesis();
        assert_eq!(genesis.height, 0);
        assert_eq!(genesis.prev_block_hash, "0");
    }

    #[test]
    fn test_block_serialization() {
        let block = Block::genesis();
        let serialized = serde_json::to_string(&block).expect("Serialization failed");
        let deserialized: Block = serde_json::from_str(&serialized).expect("Deserialization failed");

        assert_eq!(block.hash, deserialized.hash);
        assert_eq!(block.timestamp, deserialized.timestamp);
    }
}