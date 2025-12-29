use ed25519_dalek::SigningKey;
use std::fs;
use std::path::Path;
use rand::rngs::OsRng;
use std::io;

pub struct WalletManager;

impl WalletManager {
    pub fn load_or_generate(file_path: &str) -> SigningKey {
        if Path::new(file_path).exists() {
            println!("Found existing wallet at '{}'. Loading identity...", file_path);
            match Self::load(file_path) {
                Ok(key) => key,
                Err(e) => {
                    eprintln!("Failed to load wallet: {}. Backing up and generating a new one.", e);
                    let _ = fs::rename(file_path, format!("{}.bak", file_path));
                    let key = Self::generate();
                    Self::save(&key, file_path).expect("Critical: Failed to save new wallet");
                    key
                }
            }
        } else {
            println!("No wallet found. Generating new identity...");
            let key = Self::generate();
            Self::save(&key, file_path).expect("Critical: Failed to save new wallet");
            key
        }
    }

    fn generate() -> SigningKey {
        let mut csprng = OsRng;
        SigningKey::generate(&mut csprng)
    }

    fn save(key: &SigningKey, path: &str) -> io::Result<()> {
        let bytes = key.to_bytes();
        let hex_str = hex::encode(bytes);
        fs::write(path, hex_str)
    }

    fn load(path: &str) -> io::Result<SigningKey> {
        let hex_str = fs::read_to_string(path)?;
        let clean_hex = hex_str.trim();

        let bytes = hex::decode(clean_hex).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid Hex format: {}", e))
        })?;
        
        let array: [u8; 32] = bytes.try_into().map_err(|_| {
             io::Error::new(io::ErrorKind::InvalidData, "Invalid key length (expected 32 bytes)")
        })?;

        Ok(SigningKey::from_bytes(&array))
    }

    pub fn create_new_wallet(path: &str) {
        if Path::new(path).exists() {
            let _ = fs::rename(path, format!("{}.bak", path));
            println!("Existing wallet found. Backup created at '{}.bak'", path);
        }

        let key = Self::generate();
        Self::save(&key, path).expect("Failed to save wallet");
        
        let pub_key = key.verifying_key();
        let address = hex::encode(pub_key.to_bytes());
        
        println!("New wallet generated successfully!");
        println!("Saved to: {}", path);
        println!("Address: {}", address);
    }

    pub fn show_wallet_info(path: &str) {
        if !Path::new(path).exists() {
            println!("No wallet found at '{}'. Use 'wallet new' to create one.", path);
            return;
        }

        match Self::load(path) {
            Ok(key) => {
                let pub_key = key.verifying_key();
                let address = hex::encode(pub_key.to_bytes());
                println!("=== Wallet Info ===");
                println!("Path: {}", path);
                println!("Address: {}", address);
            },
            Err(e) => println!("Error loading wallet: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cleanup(path: &str) {
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(format!("{}.bak", path));
    }

    #[test]
    fn test_wallet_save_and_load() {
        let path = "test_wallet.key";
        cleanup(path);

        let original_key = WalletManager::load_or_generate(path);
        let original_bytes = original_key.to_bytes();

        let loaded_key = WalletManager::load_or_generate(path);
        let loaded_bytes = loaded_key.to_bytes();

        assert_eq!(original_bytes, loaded_bytes, "Loaded key should match saved key");

        cleanup(path);
    }

    #[test]
    fn test_corrupt_wallet_recovery() {
        let path = "test_corrupt.key";
        cleanup(path);

        fs::write(path, "ESTO_NO_ES_HEXADECIMAL").unwrap();

        let new_key = WalletManager::load_or_generate(path);
        
        assert!(Path::new(path).exists(), "New wallet file should be created");
        assert!(Path::new(&format!("{}.bak", path)).exists(), "Backup of corrupt wallet should exist");
        
        let _ = new_key.to_bytes();

        cleanup(path);
    }
}