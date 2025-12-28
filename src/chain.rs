use crate::block::Block;
use crate::transaction::Transaction;
use serde::{Serialize, Deserialize};
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};


#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub difficulty: usize,
    
    #[serde(skip)]
    pub storage_path: String,

    #[serde(skip, default)]
    pub pending_transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn new(difficulty: usize, storage_path: String) -> Blockchain {
        if std::path::Path::new(&storage_path).exists() {
            let _ = fs::remove_file(&storage_path);
        }

        let mut genesis = Block::genesis();
        genesis.difficulty = difficulty;
        genesis.hash = genesis.calculate_hash();
        genesis.mine();

        let chain = Blockchain {
            blocks: vec![genesis.clone()],
            difficulty,
            storage_path: storage_path.clone(),
            pending_transactions: Vec::new(),
        };

        let _ = chain.append_block_to_disk(&chain.blocks[0]);
    
        chain
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> bool {
        if !transaction.is_valid() {
            println!("Invalid transaction: Invalid or malformed signature.");
            return false;
        }

        self.pending_transactions.push(transaction);
        println!("Transaction added to Mempool.");
        true
    }

    pub fn mine_pending_transactions(&mut self, miner_address: String) {
        if self.pending_transactions.is_empty() {
            println!("No pending transactions to mine.");
            return;
        }

        println!("Packing {} transactions into a new block...", self.pending_transactions.len());

        let reward_tx = Transaction::new(
            "SYSTEM".to_string(),
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
                println!("Block mined successfully. Mempool cleared.");
            },
            Err(e) => eprintln!("Critical error saving block: {}", e),
        }
    }

    pub fn is_chain_valid(&self) -> bool {
        for (i, block) in self.blocks.iter().enumerate() {
            if block.calculate_hash() != block.hash {
                println!("Invalid block {}: Hash does not match data.", i);
                return false;
            }

            if i == 0 { continue; }
            
            let prev_block = &self.blocks[i - 1];
            if block.prev_block_hash != prev_block.hash {
                println!("Invalid block {}: Previous hash does not match.", i);
                return false;
            }
        }
        true
    }

    pub fn receive_block(&mut self, block: crate::block::Block) -> bool {
        let last_block = self.blocks.last().unwrap();

        if block.prev_block_hash != last_block.hash {
            println!("Block rejected: Previous hash mismatch (Possible fork).");
            return false;
        }

        if block.calculate_hash() != block.hash {
            println!("Block rejected: Invalid hash.");
            return false;
        }

        if !block.hash.starts_with(&"0".repeat(self.difficulty)) {
            println!("Block rejected: PoW too low.");
            return false;
        }

        println!("External block #{} added to the chain.", block.height);
        
        // Guardamos también el bloque recibido en disco
        if let Err(e) = self.append_block_to_disk(&block) {
            eprintln!("Error saving received block: {}", e);
        }

        self.blocks.push(block);
        self.pending_transactions.clear(); 
        
        true
    }

    pub fn append_block_to_disk(&self, block: &Block) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.storage_path)?; // Usamos self.storage_path

        let serialized = serde_json::to_string(&block)?;

        writeln!(file, "{}", serialized)?;
        Ok(())
    }

    pub fn load_chain(path: String) -> Option<Blockchain> {
        if !std::path::Path::new(&path).exists() {
            return None;
        }

        let file = fs::File::open(&path).ok()?;
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
            storage_path: path,
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

    fn cleanup(path: &str) {
        let _ = std::fs::remove_file(path);
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
        let path = "test_genesis.db";
        cleanup(path);
        let chain = Blockchain::new(1, path.to_string());
        
        assert_eq!(chain.blocks.len(), 1);
        assert_eq!(chain.blocks[0].prev_block_hash, "0");
        cleanup(path);
    }

    #[test]
    fn test_add_valid_transaction() {
        let path = "test_tx.db";
        cleanup(path);
        let mut chain = Blockchain::new(1, path.to_string());
        
        let tx = create_valid_tx(50);
        let accepted = chain.add_transaction(tx);
        
        assert!(accepted, "Valid transaction should be accepted");
        assert_eq!(chain.pending_transactions.len(), 1);
        cleanup(path);
    }

    #[test]
    fn test_mine_block() {
        let path = "test_mining.db";
        cleanup(path);
        let mut chain = Blockchain::new(1, path.to_string());
        
        let tx = create_valid_tx(10);
        chain.add_transaction(tx);

        chain.mine_pending_transactions("Miner1".to_string());

        assert_eq!(chain.blocks.len(), 2);
        assert_eq!(chain.pending_transactions.len(), 0);
        cleanup(path);
    }
    
    #[test]
    fn test_reject_invalid_signature() {
        let path = "test_sig.db";
        cleanup(path);
        let mut chain = Blockchain::new(1, path.to_string());
        
        let mut csprng = OsRng;
        let key_pair = SigningKey::generate(&mut csprng);
        let sender = hex::encode(key_pair.verifying_key().to_bytes());
        
        let mut tx = Transaction::new(sender, "Bob".to_string(), 100);
        tx.signature = None;
        
        let accepted = chain.add_transaction(tx);
        assert!(!accepted, "Unsigned transaction should be rejected");
        cleanup(path);
    }

    #[test]
    fn test_chain_integrity_tampering() {
        let path = "test_tamper.db";
        cleanup(path);
        let mut chain = Blockchain::new(1, path.to_string());
        
        let tx = create_valid_tx(100);
        chain.add_transaction(tx);
        chain.mine_pending_transactions("Miner".into());

        assert!(chain.is_chain_valid(), "Chain should be valid initially");

        let last_index = chain.blocks.len() - 1;
        chain.blocks[last_index].transactions[0].amount = 999999;

        assert!(!chain.is_chain_valid(), "Chain should be invalid after tampering");
        cleanup(path);
    }

    #[test]
    fn test_persistence_load() {
        let path = "test_persistence.db";
        cleanup(path);
        
        {
            let mut chain = Blockchain::new(1, path.to_string());
            let tx = create_valid_tx(10);
            chain.add_transaction(tx);
            chain.mine_pending_transactions("Miner".into());
        }

        let loaded_chain = Blockchain::load_chain(path.to_string()).expect("Should load chain");
        assert_eq!(loaded_chain.blocks.len(), 2, "Should recover exactly 2 blocks");
        
        cleanup(path);
    }
}