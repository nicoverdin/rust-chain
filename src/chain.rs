use crate::block::Block;
use crate::transaction::Transaction;
use serde::{Serialize, Deserialize};
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::collections::HashMap;

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

    #[serde(skip)]
    pub accounts: HashMap<String, u64>,
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

        let mut chain = Blockchain {
            blocks: vec![genesis.clone()],
            difficulty,
            storage_path: storage_path.clone(),
            pending_transactions: Vec::new(),
            accounts: HashMap::new(),
        };

        chain.update_account_state();
        let _ = chain.append_block_to_disk(&chain.blocks[0]);
    
        chain
    }

    pub fn update_account_state(&mut self) {
        self.accounts.clear();
        
        for block in &self.blocks {
            for tx in &block.transactions {
                if tx.sender != "SYSTEM" {
                    let balance = self.accounts.entry(tx.sender.clone()).or_insert(0);
                    if *balance >= tx.amount {
                        *balance -= tx.amount;
                    }
                }

                let balance = self.accounts.entry(tx.receiver.clone()).or_insert(0);
                *balance += tx.amount;
            }
        }
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        *self.accounts.get(address).unwrap_or(&0)
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> bool {
        if !transaction.is_valid() {
            println!("Invalid transaction: Invalid signature.");
            return false;
        }

        if transaction.sender != "SYSTEM" {
            let balance = self.get_balance(&transaction.sender);
            
            let pending_spend: u64 = self.pending_transactions
                .iter()
                .filter(|tx| tx.sender == transaction.sender)
                .map(|tx| tx.amount)
                .sum();

            let total_spend = transaction.amount + pending_spend;

            if balance < total_spend {
                println!("Transaction rejected: Insufficient funds. Balance: {}, Pending Spend: {}, Trying to spend: {}", 
                    balance, pending_spend, transaction.amount);
                return false;
            }
        }

        self.pending_transactions.push(transaction);
        println!("Transaction added to Mempool.");
        true
    }

    pub fn get_difficulty(&self) -> usize {
        let last_block = self.blocks.last().unwrap();
        if (!self.blocks.len().is_multiple_of(DIFFICULTY_ADJUSTMENT_INTERVAL as usize)) || self.blocks.is_empty() {
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
            if last_block.difficulty > 1 { return last_block.difficulty - 1; }
        }
        last_block.difficulty
    }

    pub fn mine_pending_transactions(&mut self, miner_address: String) {
        println!("Packing transactions into a new block...");

        let reward_tx = Transaction::new("SYSTEM".to_string(), miner_address.clone(), 50);
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
                self.update_account_state(); 
                println!("Block mined successfully (Diff: {}). Mempool cleared.", new_difficulty);
            },
            Err(e) => eprintln!("Critical error saving block: {}", e),
        }
    }

    pub fn is_chain_valid(&self) -> bool { Self::is_chain_valid_static(&self.blocks) }

    fn is_chain_valid_static(blocks: &[Block]) -> bool {
        for (i, block) in blocks.iter().enumerate() {
            if block.calculate_hash() != block.hash { return false; }
            if i == 0 { continue; }
            let prev_block = &blocks[i - 1];
            if block.prev_block_hash != prev_block.hash { return false; }
        }
        true
    }

    pub fn replace_chain(&mut self, new_blocks: Vec<Block>) -> bool {
        if new_blocks.len() <= self.blocks.len() { return false; }
        if !Self::is_chain_valid_static(&new_blocks) { return false; }
        
        println!("Consensus: The received chain is longer and valid. Replacing local chain.");
        self.blocks = new_blocks;
        self.update_account_state();
        let _ = self.save_chain_to_disk();
        true
    }

    pub fn receive_block(&mut self, block: crate::block::Block) -> bool {
        let last_block = self.blocks.last().unwrap();
        if block.prev_block_hash != last_block.hash { return false; }
        if block.calculate_hash() != block.hash { return false; }
        if !block.hash.starts_with(&"0".repeat(block.difficulty)) { return false; }
        
        println!("External block #{} added to the chain.", block.height);
        let _ = self.append_block_to_disk(&block);
        self.blocks.push(block);
        self.pending_transactions.clear(); 
        self.update_account_state();
        
        true
    }

    pub fn append_block_to_disk(&self, block: &Block) -> io::Result<()> {
        let mut file = OpenOptions::new().create(true).append(true).open(&self.storage_path)?;
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
        if !std::path::Path::new(&path).exists() { return None; }
        let file = fs::File::open(&path).ok()?;
        let reader = BufReader::new(file);
        let mut blocks = Vec::new();
        for line in reader.lines() {
            let line_content = line.ok()?;
            let block: Block = serde_json::from_str(&line_content).ok()?;
            blocks.push(block);
        }
        if blocks.is_empty() { return None; }
        let last_difficulty = blocks.last().map(|b| b.difficulty).unwrap_or(4);
        
        let mut chain = Blockchain {
            blocks,
            difficulty: last_difficulty,
            storage_path: path,
            pending_transactions: Vec::new(),
            accounts: HashMap::new(),
        };

        chain.update_account_state();
        Some(chain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Transaction;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    fn cleanup(path: &str) { let _ = std::fs::remove_file(path); }
    
    fn get_identity() -> (SigningKey, String) {
        let mut csprng = OsRng;
        let key = SigningKey::generate(&mut csprng);
        let addr = hex::encode(key.verifying_key().to_bytes());
        (key, addr)
    }

    fn create_signed_tx(sender_key: &SigningKey, sender_addr: String, amount: u64) -> Transaction {
        let mut tx = Transaction::new(sender_addr, "Bob".to_string(), amount);
        tx.sign(sender_key);
        tx
    }

    #[test]
    fn test_add_valid_transaction() {
        let path = "test_tx.db";
        cleanup(path);
        let mut chain = Blockchain::new(1, path.to_string());
        let (key, addr) = get_identity();
        
        chain.mine_pending_transactions(addr.clone());
        
        assert_eq!(chain.get_balance(&addr), 50, "Mining should give rewards");


        let tx = create_signed_tx(&key, addr, 20);
        let accepted = chain.add_transaction(tx);
        
        assert!(accepted, "Funded transaction should be accepted");
        cleanup(path);
    }

    #[test]
    fn test_insufficient_funds_rejected() {
        let path = "test_funds.db";
        cleanup(path);
        let mut chain = Blockchain::new(1, path.to_string());
        
        let (key, addr) = get_identity();

        // No minamos nada -> Saldo 0.
        let tx = create_signed_tx(&key, addr.clone(), 100);
        let accepted = chain.add_transaction(tx);

        assert!(!accepted, "Should reject transaction with insufficient funds");
        cleanup(path);
    }

    #[test]
    fn test_balance_calculation() {
        let path = "test_balance.db";
        cleanup(path);
        let mut chain = Blockchain::new(1, path.to_string());
        let (key, addr) = get_identity();

        assert_eq!(chain.get_balance(&addr), 0);

        chain.mine_pending_transactions(addr.clone());
        assert_eq!(chain.get_balance(&addr), 50);

        let tx = create_signed_tx(&key, addr.clone(), 20);
        chain.add_transaction(tx);
        
        chain.mine_pending_transactions("Miner2".into()); 

        assert_eq!(chain.get_balance(&addr), 30);
        
        cleanup(path);
    }

    #[test]
    fn test_replace_chain_consensus() {
        let path_a = "test_consensus_a.db";
        let path_b = "test_consensus_b.db";
        cleanup(path_a);
        cleanup(path_b);

        let mut chain_a = Blockchain::new(1, path_a.to_string());
        
        // Minamos bloques vacíos para aumentar altura
        chain_a.mine_pending_transactions("MinerA".into());
        chain_a.mine_pending_transactions("MinerA".into());

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
        chain_b.mine_pending_transactions("MinerB".into());

        let replaced = chain_b.replace_chain(chain_a.blocks.clone());
        assert!(!replaced);
        
        cleanup(path_a);
        cleanup(path_b);
    }

    #[test]
    fn test_dynamic_difficulty_increase() {
        let path = "test_difficulty.db";
        cleanup(path);
        let mut chain = Blockchain::new(1, path.to_string());
        
        for _ in 0..5 {
            chain.mine_pending_transactions("Miner".into());
        }
        
        chain.mine_pending_transactions("Miner".into());
        
        let last_block = chain.blocks.last().unwrap();
        assert_eq!(last_block.difficulty, 2);
        
        cleanup(path);
    }

    #[test]
    fn test_persistence_load() {
        let path = "test_persistence.db";
        cleanup(path);
        {
            let mut chain = Blockchain::new(1, path.to_string());
            chain.mine_pending_transactions("Miner".into());
        } 
        
        let loaded_chain = Blockchain::load_chain(path.to_string()).expect("Should load");
        assert_eq!(loaded_chain.blocks.len(), 2);
        cleanup(path);
    }
}