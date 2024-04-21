use crate::{get_reader, process_gen_pass, TextSignFormat};
use anyhow::Result;
use base64::engine::general_purpose::STANDARD;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, Key, KeyInit};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs;
use std::fs;
use std::io::Read;
use std::path::Path;

pub trait TextSign {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait TextVerify {
    fn verify(&self, reader: impl Read, sig: &[u8]) -> Result<bool>;
}

pub trait KeyLoader {
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized;
}

pub trait KeyGenerator {
    fn generate() -> Result<Vec<Vec<u8>>>;
}

pub struct Blake3 {
    key: [u8; 32],
}

pub struct Ed25519Signer {
    key: SigningKey,
}

pub struct Ed25519Verifier {
    key: VerifyingKey,
}

pub fn process_text_sign(input: &str, key: &str, format: TextSignFormat) -> Result<String> {
    let mut reader = get_reader(input)?;
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    // let buf = buf.trim();

    let signed = match format {
        TextSignFormat::Blake3 => {
            let singer = Blake3::load(key)?;
            singer.sign(&mut reader)?
        }
        TextSignFormat::Ed25519 => {
            let singer = Ed25519Signer::load(key)?;
            singer.sign(&mut reader)?
        }
    };

    let signed = URL_SAFE_NO_PAD.encode(signed);

    Ok(signed)
}

pub fn process_text_verify(
    input: &str,
    key: &str,
    format: TextSignFormat,
    sig: &str,
) -> Result<bool> {
    let mut reader = get_reader(input)?;
    let sig = URL_SAFE_NO_PAD.decode(sig)?;
    match format {
        TextSignFormat::Blake3 => {
            let verifier = Blake3::load(key)?;
            verifier.verify(&mut reader, &sig)
        }
        TextSignFormat::Ed25519 => {
            let verifier = Ed25519Verifier::load(key)?;
            verifier.verify(&mut reader, &sig)
        }
    }
}

pub fn process_text_generate(format: TextSignFormat) -> Result<Vec<Vec<u8>>> {
    match format {
        TextSignFormat::Blake3 => Blake3::generate(),
        TextSignFormat::Ed25519 => Ed25519Signer::generate(),
    }
}

pub fn process_text_encrypt(input: &str, key: &str) -> Result<String> {
    let mut reader = get_reader(input)?;
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    let buf = buf.trim();

    let sk = Key::from_slice(key.as_bytes()); // 32-bytes
    let cipher = ChaCha20Poly1305::new(sk);
    let nonce = [0u8; 12];

    let ciphertext = cipher.encrypt(&nonce.into(), buf.as_ref()).unwrap();
    let ciphertext = STANDARD.encode(ciphertext);

    Ok(ciphertext)
}

// for ex
pub fn process_text_decrypt(input: &str, key: &str) -> Result<String> {
    let mut reader = get_reader(input)?;
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    let buf = buf.trim();

    let ciphertext = STANDARD.decode(buf)?;

    let sk = Key::from_slice(key.as_bytes()); // 32-bytes
    let cipher = ChaCha20Poly1305::new(sk);
    let nonce = [0u8; 12];

    let decrypted = cipher.decrypt(&nonce.into(), ciphertext.as_ref()).unwrap();
    let decrypted = String::from_utf8_lossy(&decrypted);
    Ok(decrypted.to_string())
}

/// Blake3 implementation for TextVerify, KeyLoader, KeyGenerate, and TextSign

impl TextSign for Blake3 {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(blake3::keyed_hash(&self.key, &buf).as_bytes().to_vec())
    }
}

impl TextVerify for Blake3 {
    fn verify(&self, mut reader: impl Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let hash = blake3::keyed_hash(&self.key, &buf);
        let hash = hash.as_bytes();
        Ok(hash == sig)
    }
}

impl KeyLoader for Blake3 {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyGenerator for Blake3 {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let key = process_gen_pass(32, true, true, true, true)?;
        let key = key.as_bytes().to_vec();
        Ok(vec![key])
    }
}

/// Ed25519Signer implementation for TextSign and KeyLoader
impl TextSign for Ed25519Signer {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let sig = self.key.sign(&buf);
        Ok(sig.to_bytes().to_vec())
    }
}

impl KeyLoader for Ed25519Signer {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyGenerator for Ed25519Signer {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let mut csprng = rngs::OsRng;
        let sk = SigningKey::generate(&mut csprng);
        let pk = sk.verifying_key().to_bytes().to_vec();
        let sk = sk.as_bytes().to_vec();
        Ok(vec![sk, pk])
    }
}

/// Ed25519Verifier implementation for TextVerify and KeyLoader
impl TextVerify for Ed25519Verifier {
    fn verify(&self, mut reader: impl Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let sig = Signature::from_bytes(sig.try_into()?);
        let ret = self.key.verify(&buf, &sig).is_ok();
        Ok(ret)
    }
}

impl KeyLoader for Ed25519Verifier {
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl Blake3 {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = &key[..32];
        let key = key.try_into()?;
        let singer = Blake3::new(key);
        Ok(singer)
    }
}

impl Ed25519Signer {
    pub fn new(key: SigningKey) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = SigningKey::from_bytes(key.try_into()?);
        Ok(Ed25519Signer::new(key))
    }
}

impl Ed25519Verifier {
    pub fn new(key: VerifyingKey) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = VerifyingKey::from_bytes(key.try_into()?)?;
        Ok(Ed25519Verifier::new(key))
    }
}

/// test

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_sign_verify() -> Result<()> {
        let blake3 = Blake3::load("fixtures/blake3.txt")?;

        let data = b"hello";
        let sig = blake3.sign(&mut &data[..]).unwrap();

        assert!(blake3.verify(&mut &data[..], &sig).unwrap());

        Ok(())
    }

    #[test]
    fn text_ed25519_sign_verify() -> Result<()> {
        let sk = Ed25519Signer::load("fixtures/ed25519_private.txt")?;
        let pk = Ed25519Verifier::load("fixtures/ed25519_public.txt")?;

        let data = b"hello";
        let sig = sk.sign(&mut &data[..])?;

        assert!(pk.verify(&mut &data[..], &sig)?);

        Ok(())
    }

    #[test]
    fn test_encrypt_decrypt() -> Result<()> {
        let key = process_gen_pass(32, true, true, true, true)?;
        let key = key.as_bytes();
        let key = Key::from_slice(key);

        let data = "hello";
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = [0u8; 12];

        let ciphertext = cipher.encrypt(&nonce.into(), data.as_ref()).unwrap();
        let decrypted = cipher.decrypt(&nonce.into(), ciphertext.as_ref()).unwrap();
        let decrypted = String::from_utf8_lossy(&decrypted);

        assert_eq!(data, decrypted);

        Ok(())
    }
}
