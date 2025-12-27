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
        if !transaction.is_valid() {
            println!("Invalid transaction: invalid or malformed firm.");
            return false;
        }

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
            "SISTEM".to_string(),
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
            if block.calculate_hash() != block.hash {
                println!("Invalid block {}: hash and data doesn't match.", i);
                return false;
            }

            if i == 0 { continue; }
            
            let prev_block = &self.blocks[i - 1];
            if block.prev_block_hash != prev_block.hash {
                println!("Invalid block {}: Previous hash doesn't match.", i);
                return false;
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Transaction;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    fn cleanup() {
        let _ = std::fs::remove_file("history.db");
    }

    fn create_valid_tx(amount: u64) -> Transaction {
        let mut csprng = OsRng;
        let key_pair = SigningKey::generate(&mut csprng);
        let sender = hex::encode(key_pair.verifying_key().to_bytes());
        
        let mut tx = Transaction::new(sender, "Bob".to_string(), amount);
        tx.sign(&key_pair);
        tx
    }

    #[test]
    fn test_genesis_block() {
        cleanup();
        let chain = Blockchain::new(1);
        assert_eq!(chain.blocks.len(), 1);
    }

    #[test]
    fn test_add_valid_transaction() {
        cleanup();
        let mut chain = Blockchain::new(1);
        
        let tx = create_valid_tx(50);
        
        let accepted = chain.add_transaction(tx);
        
        assert!(accepted, "La transacción válida debería ser aceptada");
        assert_eq!(chain.pending_transactions.len(), 1);
    }

    #[test]
    fn test_mine_block() {
        cleanup();
        let mut chain = Blockchain::new(1);
        
        let tx = create_valid_tx(10);
        chain.add_transaction(tx);

        chain.mine_pending_transactions("Miner1".to_string());

        assert_eq!(chain.blocks.len(), 2);
        assert_eq!(chain.pending_transactions.len(), 0);
    }
    
    #[test]
    fn test_reject_invalid_signature() {
        cleanup();
        let mut chain = Blockchain::new(1);
        
        let mut csprng = OsRng;
        let key_pair = SigningKey::generate(&mut csprng);
        let sender = hex::encode(key_pair.verifying_key().to_bytes());
        
        let tx = Transaction::new(sender, "Bob".to_string(), 100);
        
        let accepted = chain.add_transaction(tx);
        assert!(!accepted, "La transacción sin firma debería ser rechazada");
    }
}