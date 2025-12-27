use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub timestamp: i64,
    pub data: String,
    pub prev_block_hash: String,
    pub hash: String,
    pub height: u64,
    pub nonce: u64
}

impl Block {
    pub fn new(data: String, prev_block_hash: String, height: u64) -> Block {
        let timestamp = Utc::now().timestamp();
        let mut block = Block {
            timestamp,
            data,
            prev_block_hash,
            hash: String::new(),
            height,
            nonce: 0,
        };
        
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let input = format!("{}{}{}{}{}", 
            self.timestamp, 
            self.data, 
            self.prev_block_hash, 
            self.height,
            self.nonce,
        );
        
        let mut hasher = Sha256::new();
        hasher.update(input);
        let result = hasher.finalize();
        
        hex::encode(result)
    }

    pub fn mine(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);

        println!("⛏️  Mining block {}...", self.height);

        while &self.hash[0..difficulty] != target {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }

        println!("✅ Block successfully mined! Nonce: {}, Hash: {}", self.nonce, self.hash);
    }

    pub fn genesis() -> Block {
        let mut block = Block::new(
            "Genesis Block".to_string(), 
            "0".to_string(),
            0
        );

        block.hash = block.calculate_hash();
        block
    }
}