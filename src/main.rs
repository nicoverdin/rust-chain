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
use tokio::sync::{Mutex, mpsc, oneshot};
use ed25519_dalek::SigningKey;
use clap::{Parser, Subcommand};
use colored::*; // ¡Colores!

const DB_PATH: &str = "history.db";
const WALLET_PATH: &str = "wallet.key";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Node,
    Wallet {
        #[command(subcommand)]
        action: WalletAction,
    },
}

#[derive(Subcommand)]
enum WalletAction {
    New,
    Show,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Wallet { action }) => {
            match action {
                WalletAction::New => {
                    WalletManager::create_new_wallet(WALLET_PATH);
                },
                WalletAction::Show => {
                    WalletManager::show_wallet_info(WALLET_PATH);
                }
            }
        },
        Some(Commands::Node) | None => {
            start_node().await;
        }
    }
}

async fn start_node() {
    println!("{}", "Initializing RustChain Node...".dimmed());

    let key_pair: SigningKey = WalletManager::load_or_generate(WALLET_PATH);
    let public_key = key_pair.verifying_key();
    let my_address = hex::encode(public_key.to_bytes());

    println!("\n{}", "=== IDENTITY LOADED ===".bold().green());
    println!("   Address: {}", my_address.cyan().bold());
    println!("   {}", "(This address is persistent)".italic().dimmed());

    println!("\n{}", format!("Loading blockchain from {}...", DB_PATH).yellow());
    let chain = Blockchain::load_chain(DB_PATH.to_string()).unwrap_or_else(|| {
        println!("   {}", "No history found. Creating Genesis block.".yellow().dimmed());
        Blockchain::new(2, DB_PATH.to_string())
    });

    let chain_shared = Arc::new(Mutex::new(chain));

    // 3. Configurar P2P
    let (p2p_sender, p2p_receiver) = mpsc::channel(32);
    let (init_sender, init_receiver) = oneshot::channel();

    let chain_for_p2p = chain_shared.clone();
    tokio::spawn(async move {
        if let Err(e) = p2p::start_network(chain_for_p2p, p2p_receiver, init_sender).await {
            eprintln!("Error in P2P Network: {}", e);
        }
    });

    println!("{}", "Starting P2P Network...".yellow());
    
    let _ = init_receiver.await; 
    
    println!("{}", "Network Active.".green().bold());

    run_user_interface(chain_shared, key_pair, my_address, p2p_sender).await;
}

async fn run_user_interface(
    chain_shared: Arc<Mutex<Blockchain>>, 
    key_pair: SigningKey, 
    my_address: String,
    p2p_sender: mpsc::Sender<NetworkMessage>
) {
    loop {
        println!("\n{}", "=== RustChain Node Menu ===".bold().blue());
        println!("1. {}", "Send Money".cyan());
        println!("2. {}", "Mine Block".yellow());
        println!("3. View Balance");
        println!("4. View Full Chain");
        println!("5. Validate Chain Integrity");
        println!("6. {}", "SIMULATE ATTACK".red().bold());
        println!("7. {}", "BROADCAST FULL CHAIN (Sync)".magenta());
        println!("8. Exit");
        print!("{}", "Select option: ".bold());
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Error reading input");

        match choice.trim() {
            "1" => {
                let mut receiver = String::new();
                let mut amount_str = String::new();

                println!("\n{}", "--- New Transaction ---".cyan());
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
                    println!("{}", "Transaction verified and added to Local Mempool.".green());
                    
                    let tx_json = serde_json::to_string(&tx).unwrap();
                    let msg = NetworkMessage::NewTransaction { data: tx_json };
                    
                    if let Err(e) = p2p_sender.send(msg).await {
                         println!("Error broadcasting: {}", e);
                    } else {
                         println!("{}", "Broadcasting transaction to peers...".dimmed());
                    }

                } else {
                    println!("{}", "Transaction rejected (Invalid signature or Insufficient Funds).".red());
                }
            },
            "2" => {
                println!("\n{}", "--- Mining ---".yellow());
                let mined_block = {
                    let mut chain = chain_shared.lock().await;
                    chain.mine_pending_transactions(my_address.clone());
                    chain.blocks.last().unwrap().clone()
                };

                let block_json = serde_json::to_string(&mined_block).unwrap();
                let msg = NetworkMessage::NewBlock { data: block_json };

                if let Err(e) = p2p_sender.send(msg).await {
                    println!("Error broadcasting block: {}", e);
                } else {
                    println!("{}", "Block propagated to the network.".dimmed());
                }
            },
            "3" => {
                let chain = chain_shared.lock().await;
                let balance = chain.get_balance(&my_address);
                println!("Your On-chain Balance: {}", format!("{}", balance).green().bold());
            },
            "4" => {
                let chain = chain_shared.lock().await;
                println!("{:#?}", *chain);
            },
            "5" => {
                println!("\nAuditing chain...");
                let chain = chain_shared.lock().await;
                if chain.is_chain_valid() {
                    println!("{}", "System integrity verified.".green());
                } else {
                    println!("{}", "RED ALERT: Chain is corrupt!".red().bold().blink());
                }
            },
            "6" => {
                println!("\nInitiating hack attempt...");
                use rand::rngs::OsRng;
                let mut rng = OsRng;
                let victim_key = SigningKey::generate(&mut rng);
                let victim_addr = hex::encode(victim_key.verifying_key().to_bytes());
                
                let mut fake_tx = Transaction::new(victim_addr, my_address.clone(), 1000);
                fake_tx.sign(&key_pair);

                let mut chain = chain_shared.lock().await;
                if chain.add_transaction(fake_tx) {
                    println!("{}", "CRITICAL: Network accepted fake transaction!".red().bold());
                } else {
                    println!("{}", "SUCCESS: Attack rejected (Signature mismatch).".green());
                }
            },
            "7" => {
                println!("{}", "Preparing to broadcast full chain...".magenta());
                let chain_data = {
                    let chain = chain_shared.lock().await;
                    serde_json::to_string(&chain.blocks).unwrap()
                };
                
                let msg = NetworkMessage::FullChain { data: chain_data };
                if let Err(e) = p2p_sender.send(msg).await {
                    println!("Error broadcasting chain: {}", e);
                } else {
                    println!("{}", "Full chain broadcasted to peers.".magenta().dimmed());
                }
            },
            "8" => {
                println!("Exiting...");
                break;
            },
            _ => println!("Invalid option"),
        }
    }
}