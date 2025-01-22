use crate::utils::{hashing, encoding};
use rand::Rng;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone)]
pub struct KeyPair {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl KeyPair {
    pub fn generate() -> Self {
        let private_key = KeyPair::generate_private_key();
        let public_key = KeyPair::generate_public_key(&private_key);
        Self {
            private_key,
            public_key,
        }
    }

    fn generate_private_key() -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let mut private_key = [0u8; 32];
        rng.fill(&mut private_key);
        private_key.to_vec()
    }

    fn generate_public_key(private_key: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(private_key);
        hasher.finalize().to_vec()
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let hash = hashing::sha256(message);
        hash.iter()
            .zip(&self.private_key)
            .map(|(h, k)| h ^ k)
            .collect()
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        let hash = hashing::sha256(message);
        signature
            .iter()
            .zip(&self.public_key)
            .all(|(s, p)| *s == (*p ^ hash[0]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = KeyPair::generate();
        assert_eq!(keypair.private_key.len(), 32);
        assert_eq!(keypair.public_key.len(), 32);
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = KeyPair::generate();
        let message = b"Test message";

        let signature = keypair.sign_message(message);
        assert!(keypair.verify_signature(message, &signature));

        let tampered_message = b"Tampered message";
        assert!(!keypair.verify_signature(tampered_message, &signature));
    }
}