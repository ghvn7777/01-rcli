use std::{collections::HashMap, io::Read};

use anyhow::{Ok, Result};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::process_genpass;

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: String,
    exp: u64,
    sub: String,
    aud: String,
}

pub fn process_jwt_sign(
    reader: &mut dyn Read,
    algorithm: Algorithm,
    iss: String,
    exp: u64,
    sub: String,
    aud: String,
) -> Result<String> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let claim = Claims { iss, exp, sub, aud };

    let header = Header::new(algorithm);

    let token = encode(&header, &claim, &EncodingKey::from_secret(&buf))?;

    Ok(token)
}

pub fn process_jwt_verify(reader: &mut dyn Read, token: String) -> Result<bool> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let algorithm: Vec<&str> = token.split('.').collect();
    // info!("algorithm: {:?}", algorithm);
    if algorithm.len() != 3 {
        return Err(anyhow::anyhow!("Invalid token"));
    }

    let json_header = String::from_utf8(base64_url::decode(&algorithm[0])?)?;
    let header = serde_json::from_str::<Header>(&json_header)?;
    // info!("header: {:?}", header);

    // 基本所有字段都不验证，只验证签名
    let mut validation = Validation::new(header.alg);
    validation.validate_aud = false;
    validation.validate_exp = false;
    validation.validate_nbf = false;
    validation.sub = None;
    validation.iss = None;

    let _token = decode::<Claims>(&token, &DecodingKey::from_secret(&buf), &validation)?;
    // info!("{:?}", _token);

    Ok(true)
}

pub fn process_jwt_secret_generate() -> Result<HashMap<&'static str, Vec<u8>>> {
    let key = process_genpass(32, true, true, true, true)?;
    let mut map = HashMap::new();

    map.insert("jwt.txt", key.as_bytes().to_vec());
    Ok(map)
}
