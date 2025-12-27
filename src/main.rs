mod block;
mod chain;

use chain::Blockchain;
use std::io::{self, Write};

fn main() {
    println!("⛓️  Iniciando RustChain...");

    let mut chain = match Blockchain::load_chain() {
        Some(c) => {
            println!("📂 Cadena cargada desde disco.");
            println!("   Altura actual: {}", c.blocks.len());
            println!("   Dificultad guardada: {}", c.difficulty);
            c
        },
        None => {
            println!("No se encontró registro. Creando nueva cadena Génesis.");
            let difficulty = 4; // Configuración inicial
            let new_chain = Blockchain::new(difficulty);
            new_chain
        }
    };

    println!("\n¿Es la cadena válida?: {}", chain.is_chain_valid());

    loop {
        println!("\nMenú:");
        println!("1. Añadir nuevo bloque");
        println!("2. Ver toda la cadena");
        println!("3. Salir");
        print!("Selecciona una opción: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Error leyendo línea");

        match choice.trim() {
            "1" => {
                print!("Introduce los datos del bloque: ");
                io::stdout().flush().unwrap();
                let mut data = String::new();
                io::stdin().read_line(&mut data).expect("Error");
                chain.add_block(data.trim().to_string());
            },
            "2" => {
                println!("{:#?}", chain);
            },
            "3" => {
                println!("Saliendo... (Los datos están guardados en history.db)");
                break;
            }
            _ => println!("Opción no válida"),
        }
    }
}