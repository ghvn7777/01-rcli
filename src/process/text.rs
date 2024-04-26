use std::{collections::HashMap, io::Read};

use anyhow::Result;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};

use crate::{process_genpass, TextSignFormat};

pub trait TextSign {
    /// Sign the data from the reader and return the signature
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait TextVerifier {
    /// Verify the data from the reader against the signature
    fn verify(&self, reader: &mut dyn Read, sig: &[u8]) -> Result<bool>;
}

pub trait TextEncrypt {
    /// Encrypt the data from the reader and return the ciphertext
    fn encrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait TextDecrypt {
    /// Decrypt the data from the reader and return the plaintext
    fn decrypt(&self, buf: &[u8]) -> Result<Vec<u8>>;
}

struct Blake3 {
    key: [u8; 32],
}

struct Ed25519Signer {
    key: SigningKey,
}

struct EncryptChaCha20 {
    key: [u8; 32],
    nonce: [u8; 12],
}

struct Ed25519Verifier {
    key: VerifyingKey,
}

/// Blake3 没有公钥私钥的概念，是对称加密，只需要一个 key，其实验证和签名都是用这个 key，做的是一样的事情
/// 都是用 key 对数据进行加密，然后对比加密后的数据是否一样
impl TextSign for Blake3 {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        // TODO: improve perf by reading in chunks
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let ret = blake3::keyed_hash(&self.key, &buf);
        Ok(ret.as_bytes().to_vec())
    }
}

impl TextVerifier for Blake3 {
    fn verify(&self, reader: &mut dyn Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        // 下面两行不能合并成一行，如果在一行直接调用 as_bytes() 时，会导致它的 temporary value 被 free
        // 但是 as_bytes() 还用了它的引用，所以编译报错
        // 总结来说如果写成 `let hash = blake3::hash(&buf).as_bytes();` 他产生的结果的生命周期在 `blake3::hash(&buf)` 执行完就结束了
        // 使用结果的引用值在后面 Ok(hash == sig) 时还在使用，hash 生命周期比 blake3::hash(&buf) 还长，编译报错
        // 写成两行第一行的 hash 可以想象成 hash 1，它只是用户可读变量名被覆盖了，实际上这个变量还在，生命周期足够长，可以想象成变成了 hash_hidden_1 变量
        // 生命周期一直到函数结束，所以后面用他的引用代码不会报错
        let hash = blake3::keyed_hash(&self.key, &buf);
        let hash = hash.as_bytes();
        Ok(hash == sig)
    }
}

/// Ed25519Signer 是非对称加密。有公钥和私钥的概念，所以签名用的 key 是不一样的
impl TextSign for Ed25519Signer {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        let signature = self.key.sign(&buf);
        Ok(signature.to_bytes().to_vec())
    }
}

impl TextVerifier for Ed25519Verifier {
    fn verify(&self, reader: &mut dyn Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        // 注意这里公钥实际上是由私钥生成的，长度 64 位
        // 下面这句 &sig[..64] 产生了一个 &[u8]，try_into() 将 &[u8] 转换为 [u8; 64]
        let sig = (&sig[..64]).try_into()?;
        let signature = Signature::from_bytes(sig);
        Ok(self.key.verify(&buf, &signature).is_ok())
    }
}

impl TextEncrypt for EncryptChaCha20 {
    fn encrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new_from_slice(&self.key)
            .map_err(|_| anyhow::anyhow!("Failed to create ChaChaPoly1305 instance"))?;
        let nonce = Nonce::from_slice(&self.nonce);

        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        let ciphertext = cipher
            .encrypt(nonce, &buf[..])
            .map_err(|_| anyhow::anyhow!("Failed to encrypt data"))?;
        Ok(ciphertext)
    }
}

impl TextDecrypt for EncryptChaCha20 {
    fn decrypt(&self, buf: &[u8]) -> Result<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new_from_slice(&self.key)
            .map_err(|_| anyhow::anyhow!("Failed to create ChaChaPoly1305 instance"))?;
        let nonce = Nonce::from_slice(&self.nonce);

        let plaintext = cipher
            .decrypt(nonce, buf)
            .map_err(|_| anyhow::anyhow!("Failed to decrypt data"))?;
        Ok(plaintext)
    }
}

impl EncryptChaCha20 {
    pub fn new(key: [u8; 32], nonce: [u8; 12]) -> Self {
        Self { key, nonce }
    }

    pub fn try_new(key_and_nonce: impl AsRef<[u8]>) -> Result<Self> {
        let key_and_nonce = key_and_nonce.as_ref();
        let key = (&key_and_nonce[..32]).try_into()?;
        let nonce = (&key_and_nonce[32..32 + 12]).try_into()?;
        Ok(Self::new(key, nonce))
    }

    fn generate() -> Result<HashMap<&'static str, Vec<u8>>> {
        let key = process_genpass(48, true, true, true, true)?;
        let mut map = HashMap::new();
        map.insert("chacha20.txt", key.as_bytes().to_vec());
        Ok(map)
    }
}

impl Blake3 {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    // AsRef 和 From 有点像
    // From<T> for U: 意思是对于 T 类型来说，我们可以把它转化成 U 类型，也就是 T -> U
    // AsRef<T> for U：意思是对于 U 类型来说，我们可以把它转化成 &T 类型，U 一般也是一个引用（不强制），也就是 U(ref) -> &T
    pub fn try_new(key: impl AsRef<[u8]>) -> Result<Self> {
        let key = key.as_ref();
        // convert &[u8] to [u8; 32]
        let key = key.try_into()?;
        Ok(Self::new(key))
    }

    fn generate() -> Result<HashMap<&'static str, Vec<u8>>> {
        let key = process_genpass(32, true, true, true, true)?;
        let mut map = HashMap::new();
        map.insert("blake3.txt", key.as_bytes().to_vec());
        Ok(map)
    }
}

impl Ed25519Signer {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = SigningKey::from_bytes(key);
        Self { key }
    }

    pub fn try_new(key: impl AsRef<[u8]>) -> Result<Self> {
        let key = key.as_ref();
        let key = (&key[..32]).try_into()?;
        Ok(Self::new(key))
    }

    fn generate() -> Result<HashMap<&'static str, Vec<u8>>> {
        let mut csprng = OsRng;
        let sk: SigningKey = SigningKey::generate(&mut csprng);
        let pk: VerifyingKey = sk.verifying_key();
        let mut map = HashMap::new();
        map.insert("ed25519_sk.txt", sk.to_bytes().to_vec());
        map.insert("ed25519_pk.txt", pk.to_bytes().to_vec());

        Ok(map)
    }
}

impl Ed25519Verifier {
    pub fn try_new(key: impl AsRef<[u8]>) -> Result<Self> {
        let key = key.as_ref();
        let key = (&key[..32]).try_into()?;
        let key = VerifyingKey::from_bytes(key)?;
        Ok(Self { key })
    }
}

pub fn process_text_encrypt(
    reader: &mut dyn Read,
    key: &[u8],
    format: TextSignFormat,
) -> Result<Vec<u8>> {
    let encrypt: Box<dyn TextEncrypt> = match format {
        TextSignFormat::ChaCha20 => Box::new(EncryptChaCha20::try_new(key)?),
        _ => anyhow::bail!("Unsupported format"),
    };

    encrypt.encrypt(reader)
}

pub fn process_text_decrypt(reader: &[u8], key: &[u8], format: TextSignFormat) -> Result<Vec<u8>> {
    let decrypt: Box<dyn TextDecrypt> = match format {
        TextSignFormat::ChaCha20 => Box::new(EncryptChaCha20::try_new(key)?),
        _ => anyhow::bail!("Unsupported format"),
    };

    decrypt.decrypt(reader)
}

pub fn process_text_sign(
    reader: &mut dyn Read,
    key: &[u8],
    format: TextSignFormat,
) -> Result<Vec<u8>> {
    let signer: Box<dyn TextSign> = match format {
        TextSignFormat::Blake3 => Box::new(Blake3::try_new(key)?),
        TextSignFormat::Ed25519 => Box::new(Ed25519Signer::try_new(key)?),
        _ => anyhow::bail!("Unsupported format"),
    };

    signer.sign(reader)
}

pub fn process_text_verify(
    reader: &mut dyn Read,
    key: &[u8],
    sig: &[u8],
    format: TextSignFormat,
) -> Result<bool> {
    let verifier: Box<dyn TextVerifier> = match format {
        TextSignFormat::Blake3 => Box::new(Blake3::try_new(key)?),
        TextSignFormat::Ed25519 => Box::new(Ed25519Verifier::try_new(key)?),
        _ => anyhow::bail!("Unsupported format"),
    };

    verifier.verify(reader, sig)
}

pub fn process_text_key_generate(format: TextSignFormat) -> Result<HashMap<&'static str, Vec<u8>>> {
    match format {
        TextSignFormat::Blake3 => Blake3::generate(),
        TextSignFormat::Ed25519 => Ed25519Signer::generate(),
        TextSignFormat::ChaCha20 => EncryptChaCha20::generate(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

    const BLAKE3_KEY: &[u8] = include_bytes!("../../fixtures/blake3.txt");
    const ED25519_SK: &[u8] = include_bytes!("../../fixtures/ed25519_sk.txt");
    const ED25519_PK: &[u8] = include_bytes!("../../fixtures/ed25519_pk.txt");

    #[test]
    fn test_blake3_sign_verify() -> Result<()> {
        let mut reader = "hello".as_bytes();
        let mut reader1 = "hello".as_bytes();
        let format = TextSignFormat::Blake3;
        let sig = process_text_sign(&mut reader, BLAKE3_KEY, format)?;
        let ret = process_text_verify(&mut reader1, BLAKE3_KEY, &sig, format)?;
        assert!(ret);
        Ok(())
    }

    #[test]
    fn test_ed25519_sign_verify() -> anyhow::Result<()> {
        let mut reader1 = "hello".as_bytes();
        let mut reader2 = "hello".as_bytes();
        let format = TextSignFormat::Ed25519;

        let sig = process_text_sign(&mut reader1, ED25519_SK, format)?;
        let encoded = URL_SAFE_NO_PAD.encode(sig);

        let decoded = URL_SAFE_NO_PAD.decode(encoded)?;
        assert!(process_text_verify(&mut reader2, ED25519_PK, &decoded, format).is_ok());

        Ok(())
    }
}
