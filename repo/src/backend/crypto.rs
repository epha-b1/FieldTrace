use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

#[derive(Clone)]
pub struct Crypto {
    cipher: Aes256Gcm,
}

impl Crypto {
    pub fn new(hex_key: &str) -> Self {
        let key_bytes = hex::decode(hex_key).expect("ENCRYPTION_KEY must be valid hex");
        assert_eq!(key_bytes.len(), 32, "ENCRYPTION_KEY must be 32 bytes (64 hex chars)");
        let cipher = Aes256Gcm::new_from_slice(&key_bytes).expect("AES key");
        Self { cipher }
    }

    pub fn encrypt(&self, plaintext: &str) -> String {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ct = self.cipher.encrypt(&nonce, plaintext.as_bytes()).expect("encrypt");
        let mut combined = nonce.to_vec();
        combined.extend(ct);
        STANDARD.encode(&combined)
    }

    pub fn decrypt(&self, encoded: &str) -> String {
        let combined = STANDARD.decode(encoded).expect("base64");
        let (nonce_bytes, ct) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let pt = self.cipher.decrypt(nonce, ct).expect("decrypt");
        String::from_utf8(pt).expect("utf8")
    }
}

pub fn mask_phone(phone: &str) -> String {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 4 {
        format!("***-***-{}", &digits[digits.len() - 4..])
    } else {
        "***-***-****".into()
    }
}
