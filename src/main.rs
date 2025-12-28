mod block;
mod chain;
mod transaction;
mod p2p;

use chain::Blockchain;
use transaction::Transaction;
use p2p::NetworkMessage;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    println!("Initializing RustChain Node...");

    let mut csprng = OsRng;
    let key_pair: SigningKey = SigningKey::generate(&mut csprng);
    let public_key = key_pair.verifying_key();
    let my_address = hex::encode(public_key.to_bytes());

    println!("\nIdentity Generated:");
    println!("   Public Address: {}", my_address);
    println!("   (Copy this address to receive funds or simulate transfers)\n");

    println!("Loading blockchain from history.db...");
    let chain = Blockchain::load_chain().unwrap_or_else(|| {
        println!("   No history found. Creating Genesis block.");
        Blockchain::new(4)
    });

    let chain_shared = Arc::new(Mutex::new(chain));

    let (p2p_sender, p2p_receiver) = mpsc::channel(32);

    let chain_for_p2p = chain_shared.clone();
    tokio::spawn(async move {
        if let Err(e) = p2p::start_network(chain_for_p2p, p2p_receiver).await {
            eprintln!("Error en P2P Network: {}", e);
        }
    });

    run_user_interface(chain_shared, key_pair, my_address, p2p_sender).await;
}

async fn run_user_interface(
    chain_shared: Arc<Mutex<Blockchain>>, 
    key_pair: SigningKey, 
    my_address: String,
    p2p_sender: mpsc::Sender<NetworkMessage>
) {
    loop {
        println!("\n=== RustChain Node Menu ===");
        println!("1. Send Money (Create Signed Tx)");
        println!("2. Mine Block (Process Mempool)");
        println!("3. View Balance (Simulated)");
        println!("4. View Full Chain");
        println!("5. Validate Chain Integrity");
        println!("6. SIMULATE ATTACK (Attempt Identity Theft)");
        println!("7. Exit");
        print!("Select option: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Error reading input");

        match choice.trim() {
            "1" => {
                let mut receiver = String::new();
                let mut amount_str = String::new();

                println!("\n--- New Transaction ---");
                println!("Sender: {} (You)", my_address);
                
                print!("Recipient (Hex Address): "); 
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut receiver).unwrap();
                
                print!("Amount: "); 
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut amount_str).unwrap();
                let amount: u64 = amount_str.trim().parse().unwrap_or(0);

                let mut tx = Transaction::new(
                    my_address.clone(), 
                    receiver.trim().to_string(), 
                    amount
                );
                tx.sign(&key_pair);

                let mut chain = chain_shared.lock().await;
                if chain.add_transaction(tx.clone()) {
                    println!("Transaction verified and added to Local Mempool.");
                    
                    let tx_json = serde_json::to_string(&tx).unwrap();
                    let msg = NetworkMessage::NewTransaction { data: tx_json };
                    
                    if let Err(e) = p2p_sender.send(msg).await {
                         println!("Error broadcasting to network: {}", e);
                    } else {
                         println!("Broadcasting transaction to peers...");
                    }

                } else {
                    println!("Network rejected the transaction.");
                }
            },
            "2" => {
                println!("\n--- Mining ---");
                let mut chain = chain_shared.lock().await;
                chain.mine_pending_transactions(my_address.clone());
            },
            "3" => {
                let mut balance: i64 = 0;
                let chain = chain_shared.lock().await;
                for block in &chain.blocks {
                    for tx in &block.transactions {
                        if tx.receiver == my_address { balance += tx.amount as i64; }
                        if tx.sender == my_address { balance -= tx.amount as i64; }
                    }
                }
                println!("Your On-chain Balance: {}", balance);
            },
            "4" => {
                let chain = chain_shared.lock().await;
                println!("{:#?}", *chain);
            },
            "5" => {
                println!("\nAuditing chain...");
                let chain = chain_shared.lock().await;
                if chain.is_chain_valid() {
                    println!("System is INTEGRAL.");
                } else {
                    println!("RED ALERT: Chain is corrupt!");
                }
            },
            "6" => {
                println!("\nInitiating hack attempt...");
                let mut rng = OsRng;
                let victim_key = SigningKey::generate(&mut rng);
                let victim_address = hex::encode(victim_key.verifying_key().to_bytes());
                
                let mut fake_tx = Transaction::new(victim_address.clone(), my_address.clone(), 1000);
                fake_tx.sign(&key_pair); // Mal firmado

                let mut chain = chain_shared.lock().await;
                if chain.add_transaction(fake_tx) {
                    println!("CRITICAL: Network accepted fake transaction!");
                } else {
                    println!("SUCCESS: Attack rejected.");
                }
            },
            "7" => break,
            _ => println!("Invalid option"),
        }
    }
}