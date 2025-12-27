mod block;
mod chain;

use chain::Blockchain;

fn main() {
    println!("Iniciando RustChain...");

    let mut chain = Blockchain::new();
    println!("✅ Cadena iniciada con bloque Génesis.");

    println!("⛏️  Minando bloque 1...");
    chain.add_block("Alice paga 50 coins a Bob".to_string());
    
    println!("⛏️  Minando bloque 2...");
    chain.add_block("Bob paga 10 coins a Charlie".to_string());

    println!("\nEstado actual de la cadena:");
    println!("{:#?}", chain);

    println!("\n🔍 Verificando integridad de la cadena...");
    println!("¿Es válida?: {}", chain.is_chain_valid());

    println!("\n😈 Intentando manipular el historial...");
    chain.blocks[1].data = "Alice paga 1000000 coins a Bob".to_string();
    
    println!("🔍 Verificando integridad tras el ataque...");
    println!("¿Es válida?: {}", chain.is_chain_valid());
}