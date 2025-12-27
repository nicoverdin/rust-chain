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
        if self.sender == "SISTEM" {
            return true
        }

        if self.signature.is_none() {
            println!("Error: transaction without signature.");
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
                println!("Error: Invalid signature for this receiver.");
                false
            },
        }
    }
}