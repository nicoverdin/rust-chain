use crate::block::Block;
use crate::transaction::Transaction;
use serde::{Serialize, Deserialize};
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};

const BLOCK_GENERATION_INTERVAL: u64 = 4;
const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 = 5;

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

    pub fn get_difficulty(&self) -> usize {
        let last_block = self.blocks.last().unwrap();
        
        if (self.blocks.len() % DIFFICULTY_ADJUSTMENT_INTERVAL as usize != 0) || self.blocks.len() == 0 {
            return last_block.difficulty;
        }

        let adjustment_block = &self.blocks[self.blocks.len() - DIFFICULTY_ADJUSTMENT_INTERVAL as usize];
        
        let time_expected = BLOCK_GENERATION_INTERVAL * DIFFICULTY_ADJUSTMENT_INTERVAL;
        let time_taken = last_block.timestamp - adjustment_block.timestamp;

        if time_taken < (time_expected as i64 / 2) {
            println!("Network is working too fast! Increasing difficulty +1");
            return last_block.difficulty + 1;
        } else if time_taken > (time_expected as i64 * 2) {
            println!("Network is too slow. Decreasing difficulty -1");
            if last_block.difficulty > 1 {
                return last_block.difficulty - 1;
            }
        }

        last_block.difficulty
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

        let new_difficulty = self.get_difficulty();

        let mut new_block = Block::new(
            block_transactions,
            prev_block.hash.clone(),
            prev_block.height + 1,
            new_difficulty,
        );

        new_block.mine();

        match self.append_block_to_disk(&new_block) {
            Ok(_) => {
                self.blocks.push(new_block);
                self.pending_transactions.clear();
                println!("Block mined successfully (Diff: {}). Mempool cleared.", new_difficulty);            },
            Err(e) => eprintln!("Critical error saving block: {}", e),
        }
    }

    pub fn is_chain_valid(&self) -> bool {
        Self::is_chain_valid_static(&self.blocks)
    }

    fn is_chain_valid_static(blocks: &[Block]) -> bool {
        for (i, block) in blocks.iter().enumerate() {
            if block.calculate_hash() != block.hash {
                println!("Invalid block {}: Hash does not match data.", i);
                return false;
            }

            if i == 0 { continue; }
            
            let prev_block = &blocks[i - 1];
            if block.prev_block_hash != prev_block.hash {
                println!("Invalid block {}: Previous hash does not match.", i);
                return false;
            }
        }
        true
    }

    pub fn replace_chain(&mut self, new_blocks: Vec<Block>) -> bool {
        if new_blocks.len() < self.blocks.len() {
            println!("Consensus: Received chain is shorter or equal. Keeping current.");
            return false;
        }

        if !Self::is_chain_valid_static(&new_blocks) {
            println!("Consensus: Received chain is invalid.");
            return false;
        }

        println!("Consensus: The received chain is longer and valid. Replacing local chain.");
        self.blocks = new_blocks;

        if let Err(e) = self.save_chain_to_disk() {
            eprintln!("Error saving new chain to disk: {}", e);
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
            .open(&self.storage_path)?;

        let serialized = serde_json::to_string(&block)?;

        writeln!(file, "{}", serialized)?;
        Ok(())
    }

    pub fn save_chain_to_disk(&self) -> io::Result<()> {
        let mut file = fs::File::create(&self.storage_path)?;

        for block in &self.blocks {
            let serialized = serde_json::to_string(&block)?;
            writeln!(file, "{}", serialized)?;
        }
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

    #[test]
    fn test_replace_chain_consensus() {
        let path_a = "test_consensus_a.db";
        let path_b = "test_consensus_b.db";
        cleanup(path_a);
        cleanup(path_b);

        let mut chain_a = Blockchain::new(1, path_a.to_string());
        
        chain_a.add_transaction(create_valid_tx(10));
        chain_a.mine_pending_transactions("MinerA".into());
        
        chain_a.add_transaction(create_valid_tx(20));
        chain_a.mine_pending_transactions("MinerA".into());

        assert_eq!(chain_a.blocks.len(), 3, "Chain A should have 3 blocks");

        let mut chain_b = Blockchain::new(1, path_b.to_string());

        let replaced = chain_b.replace_chain(chain_a.blocks.clone());

        assert!(replaced, "Node B should adopt the longer chain from A");
        assert_eq!(chain_b.blocks.len(), 3);

        cleanup(path_a);
        cleanup(path_b);
    }

    #[test]
    fn test_replace_chain_rejects_shorter() {
        let path_a = "test_short.db";
        let path_b = "test_long.db";
        cleanup(path_a);
        cleanup(path_b);

        let chain_a = Blockchain::new(1, path_a.to_string());
        
        let mut chain_b = Blockchain::new(1, path_b.to_string()); 
        chain_b.add_transaction(create_valid_tx(10));
        chain_b.mine_pending_transactions("MinerB".into());

        let replaced = chain_b.replace_chain(chain_a.blocks.clone());

        assert!(!replaced, "Node B should NOT replace its longer chain with a shorter one");
        assert_eq!(chain_b.blocks.len(), 2, "Chain B should keep its length");
        
        cleanup(path_a);
        cleanup(path_b);
    }

    #[test]
    fn test_dynamic_difficulty_increase() {
        let path = "test_difficulty.db";
        cleanup(path);
        
        let mut chain = Blockchain::new(1, path.to_string());
        
        for _ in 0..5 {
            chain.add_transaction(create_valid_tx(10));
            chain.mine_pending_transactions("Miner".into());
        }

        chain.add_transaction(create_valid_tx(10));
        chain.mine_pending_transactions("Miner".into());
        
        let last_block = chain.blocks.last().unwrap();
        assert_eq!(last_block.difficulty, 2, "Difficulty should increase because we mined too fast");
        
        cleanup(path);
    }
}