use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use crate::DbError;
use anyhow::Result;
use rand::Rng;

pub struct Crypto {
    cipher: Aes256Gcm,
}

impl Crypto {
    pub fn new(key: &[u8]) -> Result<Self> {
        if key.len() != 32 {
            return Err(DbError::Encryption("Key must be exactly 32 bytes".to_string()).into());
        }
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        Ok(Self { cipher })
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut rng = rand::thread_rng();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher
            .encrypt(nonce, data)
            .map_err(|e| DbError::Encryption(e.to_string()))?;
            
        let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 12 {
            return Err(DbError::Encryption("Invalid encrypted data".to_string()).into());
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| DbError::Encryption(e.to_string()))?;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() -> Result<()> {
        let key = [0u8; 32]; // 32-byte key filled with zeros for testing
        let crypto = Crypto::new(&key)?;
        let data = b"Hello, World!";

        let encrypted = crypto.encrypt(data)?;
        let decrypted = crypto.decrypt(&encrypted)?;

        assert_eq!(data.to_vec(), decrypted);
        Ok(())
    }

    #[test]
    fn test_invalid_key_length() {
        let key = [0u8; 16]; // Wrong key length
        assert!(Crypto::new(&key).is_err());
    }
}
