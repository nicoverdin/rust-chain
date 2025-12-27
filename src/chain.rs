use crate::block::Block;
use crate::transaction::Transaction;
use serde::{Serialize, Deserialize};
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};

const STORAGE_PATH: &str = "history.db";

#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub difficulty: usize,

    #[serde(skip, default)]
    pub pending_transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Blockchain {
        let mut genesis = Block::genesis();
        genesis.difficulty = difficulty;
        genesis.hash = genesis.calculate_hash();
        genesis.mine();

        let chain = Blockchain {
            blocks: vec![genesis.clone()],
            difficulty,
            pending_transactions: Vec::new(),
        };

        let _ = chain.append_block_to_disk(&chain.blocks[0]);
    
        chain
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> bool {
        // Future validations
        self.pending_transactions.push(transaction);
        println!("Transaction added to Mempool");
        true
    }

    pub fn mine_pending_transactions(&mut self, miner_address: String) {
        if self.pending_transactions.is_empty() {
            println!("Pending transactions is empty.");
            return;
        }

        println!("Packing {} transactions in a new block...", self.pending_transactions.len());

        // System creates money to pay the miner
        let reward_tx = Transaction::new(
            "SISTEMA".to_string(),
            miner_address,
            50,
        );
        self.pending_transactions.push(reward_tx);

        let block_transactions = self.pending_transactions.clone();

        let prev_block = self.blocks.last().unwrap();

        let mut new_block = Block::new(
            block_transactions,
            prev_block.hash.clone(),
            prev_block.height + 1,
            self.difficulty,
        );

        new_block.mine();

        match self.append_block_to_disk(&new_block) {
            Ok(_) => {
                self.blocks.push(new_block);
                self.pending_transactions.clear();
                println!("Block mined successfully and cleared Mempool.");
            },
            Err(e) => eprintln!("Critical error saving {}", e),
        }
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

    pub fn append_block_to_disk(&self, block: &Block) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(STORAGE_PATH)?;

        let serialized = serde_json::to_string(&block)?;

        writeln!(file, "{}", serialized)?;
        Ok(())
    }

    pub fn load_chain() -> Option<Blockchain> {
        if !std::path::Path::new(STORAGE_PATH).exists() {
            return None;
        }

        let file = fs::File::open(STORAGE_PATH).ok()?;
        let reader = BufReader::new(file);

        let mut blocks = Vec::new();

        for line in reader.lines() {
            let line_content = line.ok()?;
            let block: Block = serde_json::from_str(&line_content).ok()?;
            blocks.push(block);
        }

        if blocks.is_empty() {
            return None;
        }

        let last_difficulty = blocks.last().map(|b| b.difficulty).unwrap_or(4);
        Some(Blockchain {
            blocks,
            difficulty: last_difficulty,
            pending_transactions: Vec::new(),
        })
    }
}

// ... (Todo el código anterior de chain.rs)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Transaction;

    fn cleanup() {
        let _ = std::fs::remove_file("history.db");
    }

    #[test]
    fn test_genesis_block_creation() {
        cleanup();
        let chain = Blockchain::new(1);
        
        assert_eq!(chain.blocks.len(), 1);
        assert_eq!(chain.blocks[0].transactions.len(), 1);
    }

    #[test]
    fn test_add_transaction() {
        cleanup();
        let mut chain = Blockchain::new(1);
        
        let tx = Transaction::new("Alice".to_string(), "Bob".to_string(), 50);
        chain.add_transaction(tx);

        assert_eq!(chain.pending_transactions.len(), 1);
    }

    #[test]
    fn test_mine_block() {
        cleanup();
        let mut chain = Blockchain::new(1);
        
        let tx1 = Transaction::new("A".to_string(), "B".to_string(), 10);
        chain.add_transaction(tx1);

        // Minamos
        chain.mine_pending_transactions("Miner1".to_string());

        // Verificaciones
        assert_eq!(chain.blocks.len(), 2);
        assert_eq!(chain.pending_transactions.len(), 0);
        assert_eq!(chain.blocks[1].transactions.len(), 2);
    }
}