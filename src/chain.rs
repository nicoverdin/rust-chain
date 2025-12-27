use crate::block::Block;
use serde::{Serialize, Deserialize};
use std::fs;
use std::io::Write;

const STORAGE_PATH: &str = "chain.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub difficulty: usize,
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Blockchain {
        Blockchain {
            blocks: vec![Block::genesis()],
            difficulty,
        }
    }

    pub fn add_block(&mut self, data: String) {
        let prev_block = self.blocks.last().unwrap();

        let mut new_block = Block::new(
            data,
            prev_block.hash.clone(),
            prev_block.height + 1,
        );

        new_block.mine(self.difficulty);
        self.blocks.push(new_block);

        let _ = self.save_chain();
    }

    pub fn is_chain_valid(&self) -> bool {
        for (i, block) in self.blocks.iter().enumerate() {
            if i == 0 { continue; }
            let prev_block = &self.blocks[i - 1];

            if block.prev_block_hash != prev_block.hash { return false; }
            if block.calculate_hash() != block.hash { return false; }
        }
        true
    }

    pub fn save_chain(&self) -> std::io::Result<()> {
        let serialized = serde_json::to_string_pretty(&self)?;
        let mut file = fs::File::create(STORAGE_PATH)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    pub fn load_chain() -> Option<Blockchain> {
        let data = fs::read_to_string(STORAGE_PATH).ok()?;
        let chain: Blockchain = serde_json::from_str(&data).ok()?;
        Some(chain)
    }
}