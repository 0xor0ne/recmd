use chacha20poly1305::{
    aead::{
        generic_array::{typenum::U32, GenericArray},
        Aead, AeadCore, KeyInit, OsRng,
    },
    Error, XChaCha20Poly1305, XNonce,
};

pub struct Crypt {
    cipher: XChaCha20Poly1305,
}

impl Crypt {
    pub fn new(key: GenericArray<u8, U32>) -> Self {
        Crypt {
            cipher: XChaCha20Poly1305::new(&key),
        }
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<(XNonce, Vec<u8>), Error> {
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, data)?;

        Ok((nonce, ciphertext))
    }

    pub fn decrypt(&self, nonce: XNonce, data: &[u8]) -> Result<Vec<u8>, Error> {
        self.cipher.decrypt(&nonce, data)
    }
}

#[cfg(test)]
mod tests {
    use super::Crypt;
    use chacha20poly1305::{aead::generic_array::GenericArray, Error};
    #[test]
    fn encrypt_decrypt() {
        let key: [u8; 32] = [0; 32];
        let cleartext: [u8; 8] = [1; 8];
        let c = Crypt::new(GenericArray::clone_from_slice(&key));
        let (n, ct) = c.encrypt(&cleartext).unwrap();
        let output = c.decrypt(n, &ct).unwrap();

        assert!(cleartext.len() <= ct.len());
        assert_ne!(cleartext[0..8], ct[0..8]);
        assert_eq!(cleartext.len(), output.len());
        assert_eq!(cleartext[0..], output[0..]);
    }

    #[test]
    fn encrypt_corrupt_decrypt() {
        let key: [u8; 32] = [0; 32];
        let cleartext: [u8; 8] = [1; 8];
        let c = Crypt::new(GenericArray::clone_from_slice(&key));
        let (n, mut ct) = c.encrypt(&cleartext).unwrap();
        ct[0] = ct[0] + 1;
        assert_eq!(c.decrypt(n, &ct), Err(Error));
    }
}
