mod block;
mod chain;

use chain::Blockchain;
use std::time::Instant; // Para medir cuánto tarda

fn main() {
    println!("⛓️  Iniciando RustChain con PoW...");

    // Dificultad 2: Instantáneo
    // Dificultad 4: Unos milisegundos
    // Dificultad 5: Se empieza a notar (1-5 segundos)
    // Dificultad 6: Prepara el café...
    let difficulty = 5; 
    let mut chain = Blockchain::new(difficulty);

    println!("La dificultad está establecida en: {}", chain.difficulty);

    let start = Instant::now();
    
    chain.add_block("Bloque 1: Datos importantes".to_string());
    chain.add_block("Bloque 2: Más datos".to_string());

    let duration = start.elapsed();
    println!("\n⏱️ Tiempo total de minado: {:?}", duration);
    
    println!("¿Cadena válida?: {}", chain.is_chain_valid());
}