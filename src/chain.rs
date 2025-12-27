use crate::block::{self, Block};

#[derive(Debug)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Blockchain {
        Blockchain {
            blocks: vec![Block::genesis()],
        }
    }

    pub fn add_block(&mut self, data: String) {
        let prev_block = self.blocks.last().unwrap();

        let new_block = Block::new(
            data,
            prev_block.hash.clone(),
            prev_block.height + 1,
        );

        self.blocks.push(new_block);
    }

    pub fn is_chain_valid(&self) -> bool {
        for (i, block) in self.blocks.iter().enumerate() {
            if i == 0 {
                continue;
            }

            let prev_block = &self.blocks[i - 1];

            if block.prev_block_hash != prev_block.hash {
                println!("❌ Error en bloque {}: El enlace previo está roto", i);
                return false;
            }

            if block.calculate_hash() != block.hash {
                println!("❌ Error en bloque {}: El hash no coincide con los datos", i);
                return false;
            }
        }
        true
    }
}