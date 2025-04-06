use ring::aead::{self, LessSafeKey, Nonce, UnboundKey, Algorithm};
use ring::rand::SystemRandom;
use log::{info, error};

pub struct EncryptionService {
    key: LessSafeKey,
}

impl EncryptionService {
    pub fn new(key: &[u8]) -> Self {
        let algorithm = &aead::CHACHA20_POLY1305;
        let unbound_key = UnboundKey::new(algorithm, key).expect("Failed to create unbound key");
        let key = LessSafeKey::new(unbound_key);
        info!("Encryption service initialized");
        EncryptionService { key }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, aead::Error> {
        info!("Encrypting message");
        let nonce = Self::generate_nonce();
        let mut buffer = plaintext.to_vec();
        let in_out = aead::in_place_seal(
            &self.key,
            &Nonce::assume_unique_for_key(nonce),
            &[],
            &mut buffer,
            aead::Aad::empty(),
        )?;
        info!("Message encrypted successfully");
        Ok(buffer)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, aead::Error> {
        info!("Decrypting message");
        let nonce = Self::generate_nonce();
        let mut buffer = ciphertext.to_vec();
        let plaintext_len = aead::in_place_open(
            &self.key,
            &Nonce::assume_unique_for_key(nonce),
            &[],
            &mut buffer,
            aead::Aad::empty(),
        )?;
        buffer.truncate(plaintext_len);
        info!("Message decrypted successfully");
        Ok(buffer)
    }

    fn generate_nonce() -> [u8; 12] {
        let random = SystemRandom::new();
        let mut nonce = [0u8; 12];
        random.fill(&mut nonce).expect("Failed to generate nonce");
        nonce
    }
}