use super::{decrypt, EncryptingKey};
use crate::{
    dummy_rng::DummyRng,
    traits::{Decryptor, EncryptingKeypair, RandomizedDecryptor},
    Result, RsaPrivateKey,
};
use alloc::vec::Vec;
use rand_core::CryptoRng;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use zeroize::ZeroizeOnDrop;

/// Decryption key for PKCS#1 v1.5 decryption as described in [RFC8017 § 7.2].
///
/// [RFC8017 § 7.2]: https://datatracker.ietf.org/doc/html/rfc8017#section-7.2
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DecryptingKey {
    inner: RsaPrivateKey,
}

impl DecryptingKey {
    /// Create a new verifying key from an RSA public key.
    pub fn new(key: RsaPrivateKey) -> Self {
        Self { inner: key }
    }
}

impl Decryptor for DecryptingKey {
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        decrypt::<DummyRng>(None, &self.inner, ciphertext)
    }
}

impl RandomizedDecryptor for DecryptingKey {
    fn decrypt_with_rng<R: CryptoRng + ?Sized>(
        &self,
        rng: &mut R,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>> {
        decrypt(Some(rng), &self.inner, ciphertext)
    }
}

impl EncryptingKeypair for DecryptingKey {
    type EncryptingKey = EncryptingKey;
    fn encrypting_key(&self) -> EncryptingKey {
        EncryptingKey {
            inner: self.inner.clone().into(),
        }
    }
}

impl ZeroizeOnDrop for DecryptingKey {}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "serde")]
    fn test_serde() {
        use super::*;
        use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};
        use serde_test::{assert_tokens, Configure, Token};

        let mut rng = ChaCha8Rng::from_seed([42; 32]);
        let decrypting_key =
            DecryptingKey::new(RsaPrivateKey::new(&mut rng, 64).expect("failed to generate key"));

        let tokens = [
            Token::Struct {
                name: "DecryptingKey",
                len: 1,
            },
            Token::Str("inner"),
            Token::Str(concat!(
                "3054020100300d06092a864886f70d01010105000440303e020100020900ae",
                "cdb5fae1b092570203010001020869bf9ae9d6712899020500d2aaa7250205",
                "00d46b68cb020500887e253902047b4e3a4f02040991164c"
            )),
            Token::StructEnd,
        ];
        assert_tokens(&decrypting_key.readable(), &tokens);
    }
}
