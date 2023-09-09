//! Encryption of byte streams based on a random key.

use aes::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};

type Aes128Ctr64BE = ctr::Ctr64BE<aes::Aes128>;

// Note: The KDF salt is public, as it needs to be used from the web client. It
// only exists to make rainbow table attacks less likely.
const SALT: &str =
    "This is a non-random salt for sshx.io, since we want to stretch the security of 83-bit keys!";

/// Encrypts byte streams using the Argon2 hash of a random key.
#[derive(Clone)]
pub struct Encrypt {
    aes_key: [u8; 16], // 16-bit
}

impl Encrypt {
    /// Construct a new encryptor.
    pub fn new(key: &str) -> Self {
        use argon2::{Algorithm, Argon2, Params, Version};
        // These parameters must match the browser implementation.
        let hasher = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(19 * 1024, 2, 1, Some(16)).unwrap(),
        );
        let mut aes_key = [0; 16];
        hasher
            .hash_password_into(key.as_bytes(), SALT.as_bytes(), &mut aes_key)
            .expect("failed to hash key with argon2");
        Self { aes_key }
    }

    /// Get the encrypted zero block.
    pub fn zeros(&self) -> Vec<u8> {
        let mut zeros = [0; 16];
        let mut cipher = Aes128Ctr64BE::new(&self.aes_key.into(), &zeros.into());
        cipher.apply_keystream(&mut zeros);
        zeros.to_vec()
    }

    /// Encrypt a segment of data from a stream.
    ///
    /// Note that in CTR mode, the encryption operation is the same as the
    /// decryption operation.
    pub fn segment(&self, stream_num: u64, offset: u64, data: &[u8]) -> Vec<u8> {
        assert_ne!(stream_num, 0, "stream number must be nonzero"); // security check

        let mut iv = [0; 16];
        iv[0..8].copy_from_slice(&stream_num.to_be_bytes());

        let mut cipher = Aes128Ctr64BE::new(&self.aes_key.into(), &iv.into());
        let mut buf = data.to_vec();
        cipher.seek(offset);
        cipher.apply_keystream(&mut buf);
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::Encrypt;

    #[test]
    fn make_encrypt() {
        let encrypt = Encrypt::new("test");
        assert_eq!(
            encrypt.zeros(),
            [198, 3, 249, 238, 65, 10, 224, 98, 253, 73, 148, 1, 138, 3, 108, 143],
        );
    }

    #[test]
    fn roundtrip_ctr() {
        let encrypt = Encrypt::new("this is a test key");
        let data = b"hello world";
        let encrypted = encrypt.segment(1, 0, data);
        assert_eq!(encrypted.len(), data.len());
        let decrypted = encrypt.segment(1, 0, &encrypted);
        assert_eq!(decrypted, data);
    }

    #[test]
    fn matches_offset() {
        let encrypt = Encrypt::new("this is a test key");
        let data = b"1st block.(16B)|2nd block......|3rd block";
        let encrypted = encrypt.segment(1, 0, data);
        assert_eq!(encrypted.len(), data.len());
        for i in 1..data.len() {
            let encrypted_suffix = encrypt.segment(1, i as u64, &data[i..]);
            assert_eq!(encrypted_suffix, &encrypted[i..]);
        }
    }

    #[test]
    #[should_panic]
    fn zero_stream_num() {
        let encrypt = Encrypt::new("this is a test key");
        encrypt.segment(0, 0, b"hello world");
    }
}
