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
        };
        
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let input = format!("{}{}{}{}", 
            self.timestamp, 
            self.data, 
            self.prev_block_hash, 
            self.height
        );
        
        let mut hasher = Sha256::new();
        hasher.update(input);
        let result = hasher.finalize();
        
        hex::encode(result)
    }

    pub fn genesis() -> Block {
        Block::new(
            "Genesis Block".to_string(), 
            "0".to_string(),
            0
        )
    }
}