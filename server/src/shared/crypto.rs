use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use rand::RngCore;

const NONCE_LEN: usize = 12;

pub fn encrypt(key: &[u8; 32], plaintext: &str) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key.into());

    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from(nonce_bytes);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .expect("AES-GCM encryption failed");

    [nonce.as_slice(), &ciphertext].concat()
}

pub fn decrypt(key: &[u8; 32], data: &[u8]) -> Option<String> {
    if data.len() < NONCE_LEN {
        return None;
    }
    let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
    let nonce_bytes: [u8; NONCE_LEN] = nonce_bytes.try_into().ok()?;

    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from(nonce_bytes);

    let plaintext = cipher.decrypt(&nonce, ciphertext).ok()?;
    String::from_utf8(plaintext).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: [u8; 32] = [7; 32];

    #[test]
    fn decrypt_reverses_encrypt() {
        let encrypted = encrypt(&KEY, "gho_secret_token");
        assert_eq!(
            decrypt(&KEY, &encrypted),
            Some("gho_secret_token".to_string())
        );
    }

    #[test]
    fn decrypt_fails_with_wrong_key() {
        let encrypted = encrypt(&KEY, "gho_secret_token");
        let wrong_key = [9; 32];
        assert_eq!(decrypt(&wrong_key, &encrypted), None);
    }

    #[test]
    fn decrypt_fails_for_truncated_data() {
        assert_eq!(decrypt(&KEY, b"too short"), None);
    }

    #[test]
    fn encrypt_output_is_not_the_plaintext() {
        let encrypted = encrypt(&KEY, "gho_secret_token");
        assert!(!encrypted.windows(4).any(|w| w == b"gho_"));
    }
}
