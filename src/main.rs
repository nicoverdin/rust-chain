mod block;
use block::Block;

fn main() {
    println!("Iniciando RustChain...");

    let genesis = Block::genesis();
    println!("Bloque Génesis minado: {}...", &genesis.hash[0..10]);
    println!("{:#?}", genesis);

    let block_2 = Block::new(
        "Transacción: Alice paga a Bob 5 BTC".to_string(),
        genesis.hash.clone(),
        1
    );
    
    println!("\nNuevo Bloque añadido: {}...", &block_2.hash[0..10]);
    println!("Hash previo: {}...", &block_2.prev_block_hash[0..10]);
    println!("{:#?}", block_2);
}