use crate::block::Block;
use serde::{Serialize, Deserialize};
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};

const STORAGE_PATH: &str = "history.db";

#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub difficulty: usize,
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Blockchain {
        let mut genesys = Block::genesis();
        genesys.difficulty = difficulty;
        genesys.hash = genesys.calculate_hash();
        genesys.mine();

        let chain = Blockchain {
            blocks: vec![genesys.clone()],
            difficulty,
        };

        let _ = chain.append_block_to_disk(&chain.blocks[0]);
    
        chain
    }

    pub fn add_block(&mut self, data: String) {
        let prev_block = self.blocks.last().unwrap();

        let mut new_block = Block::new(
            data,
            prev_block.hash.clone(),
            prev_block.height + 1,
            self.difficulty,
        );

        new_block.mine();

        match self.append_block_to_disk(&new_block) {
            Ok(_) => {
                self.blocks.push(new_block);
                println!("💾 Bloque guardado en disco (Append-Only)");
            },
            Err(e) => eprintln!("❌ Error crítico guardando bloque en disco: {}", e),
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
        })
    }
}