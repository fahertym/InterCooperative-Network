use sha2::{Sha256, Digest};

pub fn hex_encode(data: &[u8]) -> String {
    hex::encode(data)
}

pub fn hex_decode(s: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(s)
}

pub fn hash_data(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

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

pub mod time {
    use chrono::{DateTime, Utc, Duration};

    pub fn now() -> DateTime<Utc> {
        Utc::now()
    }

    pub fn is_expired(timestamp: DateTime<Utc>, duration: Duration) -> bool {
        now() > timestamp + duration
    }
}

pub mod crypto {
    use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
    use rand::rngs::OsRng;

    pub fn generate_keypair() -> Keypair {
        let mut csprng = OsRng{};
        Keypair::generate(&mut csprng)
    }

    pub fn sign(secret_key: &SecretKey, message: &[u8]) -> Signature {
        let keypair = Keypair {
            public: PublicKey::from(secret_key),
            secret: *secret_key,
        };
        keypair.sign(message)
    }

    pub fn verify(public_key: &PublicKey, message: &[u8], signature: &Signature) -> bool {
        public_key.verify(message, signature).is_ok()
    }
}