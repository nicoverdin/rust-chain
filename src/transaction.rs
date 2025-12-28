use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use ed25519_dalek::{Signer, Verifier, Signature, SigningKey, VerifyingKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub timestamp: u64,
    pub signature: Option<String>,
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: u64) -> Transaction {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let mut tx = Transaction {
            id: String::new(),
            sender,
            receiver,
            amount,
            timestamp,
            signature: None,
        };

        tx.id = tx.calculate_hash();
        tx
    }

    pub fn calculate_hash(&self) -> String {
        let input = format!("{}{}{}{}", 
            self.sender, 
            self.receiver, 
            self.amount, 
            self.timestamp
        );
        
        let mut hasher = Sha256::new();
        hasher.update(input);
        hex::encode(hasher.finalize())
    }

    pub fn sign(&mut self, key_pair: &SigningKey) {
        let message = self.calculate_hash();
        let signature: Signature = key_pair.sign(message.as_bytes());

        self.signature = Some(hex::encode(signature.to_bytes()));
    }

    pub fn is_valid(&self) -> bool {
        if self.sender == "SYSTEM" {
            return true
        }

        if self.signature.is_none() {
            println!("Error: Transaction without signature.");
            return false;
        }

        let public_key_bytes = match hex::decode(&self.sender) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };

        let public_key: VerifyingKey = match VerifyingKey::from_bytes(&public_key_bytes.try_into().unwrap()) {
            Ok(pk) => pk,
            Err(_) => return false,
        };

        let signature_hex = self.signature.as_ref().unwrap();
        let signature_bytes = match hex::decode(signature_hex) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };

        let signature_array: [u8; 64] = match signature_bytes.try_into() {
            Ok(arr) => arr,
            Err(_) => return false,
        };

        let signature = Signature::from_bytes(&signature_array);

        let message = self.calculate_hash();
        match public_key.verify(message.as_bytes(), &signature) {
            Ok(_) => true,
            Err(_) => {
                println!("Error: Invalid signature.");
                false
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;
    use ed25519_dalek::SigningKey;

    #[test]
    fn test_sign_and_verify_transaction() {
        let mut csprng = OsRng;
        let key_pair = SigningKey::generate(&mut csprng);
        let public_key = key_pair.verifying_key();
        let sender_address = hex::encode(public_key.to_bytes());

        let mut tx = Transaction::new(sender_address, "Receiver".to_string(), 100);

        tx.sign(&key_pair);

        assert!(tx.is_valid(), "Signed transaction should be valid");
    }

    #[test]
    fn test_tampering_transaction() {
        let mut csprng = OsRng;
        let key_pair = SigningKey::generate(&mut csprng);
        let sender_address = hex::encode(key_pair.verifying_key().to_bytes());

        let mut tx = Transaction::new(sender_address, "Receiver".to_string(), 100);
        tx.sign(&key_pair);

        tx.amount = 1000; 

        assert!(!tx.is_valid(), "Tampered transaction should be invalid");
    }

    #[test]
    fn test_system_transaction_validity() {
        let tx = Transaction::new("SYSTEM".to_string(), "Miner".to_string(), 50);
        assert!(tx.is_valid(), "SYSTEM transaction should be valid without signature");
    }

    #[test]
    fn test_missing_signature_fails() {
        let mut csprng = OsRng;
        let key_pair = SigningKey::generate(&mut csprng);
        let sender_address = hex::encode(key_pair.verifying_key().to_bytes());

        let tx = Transaction::new(sender_address, "Receiver".to_string(), 100);
        
        assert!(!tx.is_valid(), "Transaction without signature should be invalid");
    }

    #[test]
    fn test_wrong_signer_rejected() {
        let mut csprng = OsRng;
        
        let key_pair_a = SigningKey::generate(&mut csprng);
        let sender_address = hex::encode(key_pair_a.verifying_key().to_bytes());

        let key_pair_b = SigningKey::generate(&mut csprng);

        let mut tx = Transaction::new(sender_address, "Receiver".to_string(), 100);
        
        tx.sign(&key_pair_b);

        assert!(!tx.is_valid(), "Signature must match sender public key");
    }

    #[test]
    fn test_malformed_sender_address() {
        let tx = Transaction::new("INVALID_HEX_STRING".to_string(), "Receiver".to_string(), 100);
        assert!(!tx.is_valid(), "Malformed sender address should result in invalid tx");
    }

    #[test]
    fn test_malformed_signature_hex() {
        let mut csprng = OsRng;
        let key_pair = SigningKey::generate(&mut csprng);
        let sender_address = hex::encode(key_pair.verifying_key().to_bytes());

        let mut tx = Transaction::new(sender_address, "Receiver".to_string(), 100);
        tx.sign(&key_pair);

        tx.signature = Some("ZZZZZZZ_NOT_HEX".to_string());

        assert!(!tx.is_valid(), "Malformed signature hex should result in invalid tx");
    }
}