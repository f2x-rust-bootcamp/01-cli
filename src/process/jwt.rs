use crate::get_reader;
use anyhow::Result;
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use sha2::Sha256;
use std::collections::BTreeMap;

pub fn process_jwt_sign(sub: &str, aud: &str, exp: &str, key: &str) -> Result<String> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(key.as_bytes()).unwrap();
    let mut claims = BTreeMap::new();
    claims.insert("sub", sub);
    claims.insert("aud", aud);
    claims.insert("exp", exp);

    let token_str = claims.sign_with_key(&key).unwrap();

    Ok(token_str)
}

pub fn process_jwt_verify(input: &str, key: &str) -> Result<String> {
    let mut reader = get_reader(input)?;
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    let buf = buf.trim();

    let key: Hmac<Sha256> = Hmac::new_from_slice(key.as_bytes()).unwrap();

    let claims: BTreeMap<String, String> = buf.verify_with_key(&key).unwrap();

    let claims = serde_json::to_string(&claims).unwrap();

    Ok(claims)
}
