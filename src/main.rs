mod block;
mod chain;
mod transaction; // No olvides esto

use chain::Blockchain;
use transaction::Transaction;
use std::io::{self, Write};

fn main() {
    println!("⛓️  Iniciando RustChain v2 (Transaction Support)...");
    
    let mut chain = match Blockchain::load_chain() {
        Some(c) => c,
        None => {
            let difficulty = 4;
            let new_chain = Blockchain::new(difficulty);
            new_chain
        }
    };

    println!("\n¿Es la cadena válida?: {}", chain.is_chain_valid());

    loop {
        println!("\nMenú:");
        println!("1. Nueva Transacción (Minar bloque)");
        println!("2. Ver cadena");
        println!("3. Salir");
        print!("Opción: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Error");

        match choice.trim() {
            "1" => {
                let mut sender = String::new();
                let mut receiver = String::new();
                let mut amount_str = String::new();

                print!("Remitente: "); io::stdout().flush().unwrap();
                io::stdin().read_line(&mut sender).unwrap();
                
                print!("Destinatario: "); io::stdout().flush().unwrap();
                io::stdin().read_line(&mut receiver).unwrap();
                
                print!("Cantidad: "); io::stdout().flush().unwrap();
                io::stdin().read_line(&mut amount_str).unwrap();
                let amount: u64 = amount_str.trim().parse().unwrap_or(0);

                let tx = Transaction::new(
                    sender.trim().to_string(), 
                    receiver.trim().to_string(), 
                    amount
                );

                chain.add_block(vec![tx]);
            },
            "2" => println!("{:#?}", chain),
            "3" => break,
            _ => println!("Opción inválida"),
        }
    }
}