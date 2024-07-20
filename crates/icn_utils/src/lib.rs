// File: icn_utils/src/lib.rs

// Import necessary modules and crates
use sha2::{Digest, Sha256};
use chrono::{DateTime, Duration, Utc};
use hex::FromHexError;

// Function to encode data to a hexadecimal string
pub fn hex_encode(data: &[u8]) -> String {
    hex::encode(data)
}

// Function to decode a hexadecimal string to data
pub fn hex_decode(s: &str) -> Result<Vec<u8>, FromHexError> {
    hex::decode(s)
}

// Function to hash data using SHA-256
pub fn hash_data(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

// Function to calculate the Merkle root from a list of hashes
pub fn calculate_merkle_root(hashes: &[Vec<u8>]) -> Vec<u8> {
    if hashes.is_empty() {
        return vec![];
    }
    if hashes.len() == 1 {
        return hashes[0].clone();
    }
    let mut next_level = Vec::new();
    for chunk in hashes.chunks(2) {
        let mut hasher = Sha256::new();
        hasher.update(&chunk[0]);
        if chunk.len() > 1 {
            hasher.update(&chunk[1]);
        } else {
            hasher.update(&chunk[0]);
        }
        next_level.push(hasher.finalize().to_vec());
    }
    calculate_merkle_root(&next_level)
}

// Time-related utilities
pub mod time {
    use super::*;

    // Function to get the current UTC time
    pub fn now() -> DateTime<Utc> {
        Utc::now()
    }

    // Function to check if a given timestamp is expired based on a duration
    pub fn is_expired(timestamp: DateTime<Utc>, duration: Duration) -> bool {
        now() > timestamp + duration
    }
}

// Cryptographic utilities
pub mod crypto {
    use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
    use rand::rngs::OsRng;

    // Function to generate a new key pair (public and private keys)
    pub fn generate_keypair() -> Keypair {
        let mut csprng = OsRng {};
        Keypair::generate(&mut csprng)
    }

    // Function to sign a message with a given secret key
    pub fn sign(secret_key: &SecretKey, message: &[u8]) -> Signature {
        let keypair = Keypair {
            public: PublicKey::from(secret_key),
            secret: *secret_key,
        };
        keypair.sign(message)
    }

    // Function to verify a signature with a given public key and message
    pub fn verify(public_key: &PublicKey, message: &[u8], signature: &Signature) -> bool {
        public_key.verify(message, signature).is_ok()
    }
}

// Unit tests for the utility functions
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_encode_decode() {
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let encoded = hex_encode(&data);
        assert_eq!(encoded, "12345678");
        let decoded = hex_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_hash_data() {
        let data = b"test data";
        let hash = hash_data(data);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_calculate_merkle_root() {
        let hashes = vec![
            vec![1; 32],
            vec![2; 32],
            vec![3; 32],
            vec![4; 32],
        ];
        let root = calculate_merkle_root(&hashes);
        assert_eq!(root.len(), 32);

        // Test with odd number of hashes
        let odd_hashes = vec![
            vec![1; 32],
            vec![2; 32],
            vec![3; 32],
        ];
        let odd_root = calculate_merkle_root(&odd_hashes);
        assert_eq!(odd_root.len(), 32);
    }

    #[test]
    fn test_time_utils() {
        let now = time::now();
        let duration = Duration::seconds(10);
        assert!(!time::is_expired(now, duration));
        assert!(time::is_expired(now - Duration::seconds(20), duration));
    }

    #[test]
    fn test_crypto_utils() {
        let keypair = crypto::generate_keypair();
        let message = b"test message";
        let signature = crypto::sign(&keypair.secret, message);
        assert!(crypto::verify(&keypair.public, message, &signature));
        
        // Test with incorrect message
        let wrong_message = b"wrong message";
        assert!(!crypto::verify(&keypair.public, wrong_message, &signature));
    }
}
