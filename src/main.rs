mod block;
mod chain;
mod transaction;
mod p2p;
mod wallet;

use chain::Blockchain;
use transaction::Transaction;
use wallet::WalletManager;
use p2p::NetworkMessage;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use ed25519_dalek::SigningKey;

const DB_PATH: &str = "history.db";
const WALLET_PATH: &str = "wallet.key";

#[tokio::main]
async fn main() {
    env_logger::init();
    
    println!("Initializing RustChain Node...");

    let key_pair: SigningKey = WalletManager::load_or_generate(WALLET_PATH);
    let public_key = key_pair.verifying_key();
    let my_address = hex::encode(public_key.to_bytes());

    println!("\n=== IDENTITY LOADED ===");
    println!("   Public Address: {}", my_address);
    println!("   (This address is persistent)\n");

    println!("Loading blockchain from {}...", DB_PATH);
    let chain = Blockchain::load_chain(DB_PATH.to_string()).unwrap_or_else(|| {
        println!("   No history found. Creating Genesis block.");
        Blockchain::new(2, DB_PATH.to_string())
    });

    let chain_shared = Arc::new(Mutex::new(chain));

    let (p2p_sender, p2p_receiver) = mpsc::channel(32);

    let chain_for_p2p = chain_shared.clone();
    tokio::spawn(async move {
        if let Err(e) = p2p::start_network(chain_for_p2p, p2p_receiver).await {
            eprintln!("Error in P2P Network: {}", e);
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
        println!("7. BROADCAST FULL CHAIN (Force Sync)"); // NUEVA OPCIÓN
        println!("8. Exit");
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
                         println!("Error broadcasting: {}", e);
                    }
                }
            },
            "2" => {
                println!("\n--- Mining ---");
                let mined_block = {
                    let mut chain = chain_shared.lock().await;
                    chain.mine_pending_transactions(my_address.clone());
                    chain.blocks.last().unwrap().clone()
                };

                println!("Block #{} mined locally.", mined_block.height);

                let block_json = serde_json::to_string(&mined_block).unwrap();
                let msg = NetworkMessage::NewBlock { data: block_json };

                if let Err(e) = p2p_sender.send(msg).await {
                    println!("Error broadcasting block: {}", e);
                } else {
                    println!("Block propagated to the network.");
                }
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
                let chain = chain_shared.lock().await;
                if chain.is_chain_valid() {
                    println!("System integrity verified.");
                } else {
                    println!("RED ALERT: Chain is corrupt!");
                }
            },
            "6" => {
                use rand::rngs::OsRng;
                let mut rng = OsRng;
                let victim_key = SigningKey::generate(&mut rng);
                let victim_addr = hex::encode(victim_key.verifying_key().to_bytes());
                let mut fake_tx = Transaction::new(victim_addr, my_address.clone(), 1000);
                fake_tx.sign(&key_pair); 
                let mut chain = chain_shared.lock().await;
                if chain.add_transaction(fake_tx) {
                    println!("CRITICAL: Network accepted fake transaction!");
                } else {
                    println!("SUCCESS: Attack rejected.");
                }
            },
            "7" => {
                println!("Preparing to broadcast full chain...");
                let chain_data = {
                    let chain = chain_shared.lock().await;
                    serde_json::to_string(&chain.blocks).unwrap()
                };
                
                let msg = NetworkMessage::FullChain { data: chain_data };
                if let Err(e) = p2p_sender.send(msg).await {
                    println!("Error broadcasting chain: {}", e);
                } else {
                    println!("Full chain broadcasted to peers.");
                }
            },
            "8" => break,
            _ => println!("Invalid option"),
        }
    }
}