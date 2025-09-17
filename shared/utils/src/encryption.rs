use base64::{Engine as _, engine::general_purpose};
use sha2::{Digest, Sha256};
use std::fmt;

pub struct HashUtils;

impl HashUtils {
    /// Generates SHA-256 hash of input string
    pub fn sha256(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    /// Generates SHA-256 hash and returns as base64
    pub fn sha256_base64(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        general_purpose::STANDARD.encode(result)
    }

    /// Verifies if input matches the given hash
    pub fn verify_sha256(input: &str, hash: &str) -> bool {
        Self::sha256(input) == hash
    }
}

pub struct Base64Utils;

impl Base64Utils {
    /// Encodes string to base64
    pub fn encode(input: &str) -> String {
        general_purpose::STANDARD.encode(input.as_bytes())
    }

    /// Decodes base64 string
    pub fn decode(input: &str) -> Result<String, base64::DecodeError> {
        let decoded = general_purpose::STANDARD.decode(input)?;
        String::from_utf8(decoded).map_err(|_| base64::DecodeError::InvalidByte(0, 0))
    }

    /// Encodes bytes to base64
    pub fn encode_bytes(input: &[u8]) -> String {
        general_purpose::STANDARD.encode(input)
    }

    /// Decodes base64 to bytes
    pub fn decode_bytes(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
        general_purpose::STANDARD.decode(input)
    }
}

/// Simple encryption/decryption using XOR (for demonstration purposes)
/// In production, use proper encryption libraries like AES
pub struct SimpleEncryption;

impl SimpleEncryption {
    /// XOR encryption/decryption (same operation for both)
    pub fn xor_cipher(data: &str, key: &str) -> String {
        let key_bytes = key.as_bytes();
        let key_len = key_bytes.len();
        
        data.bytes()
            .enumerate()
            .map(|(i, byte)| byte ^ key_bytes[i % key_len])
            .collect::<Vec<u8>>()
            .iter()
            .map(|&b| format!("{:02x}", b))
            .collect()
    }

    /// Decrypt XOR cipher
    pub fn xor_decipher(encrypted_hex: &str, key: &str) -> Result<String, String> {
        let key_bytes = key.as_bytes();
        let key_len = key_bytes.len();
        
        // Convert hex string to bytes
        let encrypted_bytes: Result<Vec<u8>, _> = (0..encrypted_hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&encrypted_hex[i..i + 2], 16))
            .collect();
            
        match encrypted_bytes {
            Ok(bytes) => {
                let decrypted: Vec<u8> = bytes
                    .iter()
                    .enumerate()
                    .map(|(i, &byte)| byte ^ key_bytes[i % key_len])
                    .collect();
                    
                String::from_utf8(decrypted).map_err(|e| e.to_string())
            }
            Err(e) => Err(e.to_string())
        }
    }
}

#[derive(Debug)]
pub struct EncryptionError {
    pub message: String,
}

impl fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Encryption error: {}", self.message)
    }
}

impl std::error::Error for EncryptionError {}