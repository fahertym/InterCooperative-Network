use chrono::{DateTime, Utc};
use ed25519_dalek::{Keypair, PublicKey, Signature, Verifier};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecentralizedIdentity {
    pub id: String,
    #[serde(with = "public_key_serde")]
    pub public_key: PublicKey,
    pub created_at: DateTime<Utc>,
    pub reputation: f64,
    pub attributes: HashMap<String, String>,
}

mod public_key_serde {
    use ed25519_dalek::PublicKey;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(public_key: &PublicKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = public_key.to_bytes();
        bytes.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        PublicKey::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}

impl DecentralizedIdentity {
    pub fn new(attributes: HashMap<String, String>) -> (Self, Keypair) {
        let mut csprng = rand::rngs::OsRng {};
        let keypair: Keypair = Keypair::generate(&mut csprng);
        let public_key = keypair.public;
        let id = format!("did:icn:{}", hex::encode(public_key.to_bytes()));

        (
            Self {
                id,
                public_key,
                created_at: Utc::now(),
                reputation: 1.0,
                attributes,
            },
            keypair,
        )
    }

    pub fn verify_signature(&self, message: &[u8], signature: &Signature) -> bool {
        self.public_key.verify(message, signature).is_ok()
    }
}