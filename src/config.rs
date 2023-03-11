use sha2::{
    digest::crypto_common::generic_array::{typenum::U32, GenericArray},
    Digest, Sha256,
};
use std::time::Duration;

const TCP_CONNECT_TO: Duration = Duration::new(5, 0);
const TCP_RESP_TO: Duration = Duration::new(30, 0);
const TCP_WRITE_TO: Duration = Duration::new(10, 0);
const HISTORY_DEPTH: usize = 100_000;
const PASSWORD_DEF: &str = "1e$tob5UtRi6oFr8jlYO";
// SHA256 of "RECMDK"
const EK: &str = "e3457b2c5a7614014dcc1123e35479a10b284ae06340162f0b616948bdd33535";

#[derive(Debug)]
pub struct Config {
    key: GenericArray<u8, U32>,
    history_depth: usize,
    tcp_connect_to: Duration,
    tcp_write_to: Duration,
    tcp_resp_to: Duration,
}

impl Config {
    pub fn init() -> Self {
        let key = Config::init_key(EK);

        Config {
            key,
            history_depth: HISTORY_DEPTH,
            tcp_connect_to: TCP_CONNECT_TO,
            tcp_write_to: TCP_WRITE_TO,
            tcp_resp_to: TCP_RESP_TO,
        }
    }

    fn init_key(s: &str) -> GenericArray<u8, U32> {
        let mut pwd = String::from(PASSWORD_DEF);

        let eka = GenericArray::clone_from_slice(&hex::decode(s).unwrap());

        for (k, v) in std::env::vars() {
            let mut hasher = Sha256::new();
            hasher.update(&k);
            let ksha256 = hasher.finalize();

            if eka == ksha256 {
                println!("FOUND");
                pwd = v;
                std::env::set_var(k, "");
                break;
            }
        }

        let mut hasher = Sha256::new();
        hasher.update(pwd.as_bytes());
        hasher.finalize()
    }

    pub fn get_key(&self) -> GenericArray<u8, U32> {
        self.key
    }

    pub fn get_history_depth(&self) -> usize {
        self.history_depth
    }

    pub fn get_tcp_connect_to(&self) -> Duration {
        self.tcp_connect_to
    }

    pub fn get_tcp_write_to(&self) -> Duration {
        self.tcp_write_to
    }

    pub fn get_tcp_resp_to(&self) -> Duration {
        self.tcp_resp_to
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use sha2::digest::crypto_common::generic_array::GenericArray;
    #[test]
    fn init_and_check_len() {
        let c = Config::init();
        assert_eq!(c.get_key().len(), 32);
    }

    #[test]
    fn init_key_default() {
        // SHA256 of non existing nv variable name
        let eks = "0000000000000000000000000000000000000000000000000000000000000000";
        // SHA256 of default password
        let expected = "ce5445b88cdccca025bfdc830363de36e4863141da0a58e5ad7b31ee4b2b67d1";
        let expected_ga = GenericArray::clone_from_slice(&hex::decode(expected).unwrap());

        let key = Config::init_key(eks);

        assert_eq!(expected_ga, key);
    }

    #[test]
    fn init_key_env() {
        // SHA256 of "AAAABBBBCCCC"
        let eks = "97b9883915d85cfdd180ef552b68a583a706e6deaf49dc56353dd058e2a8b2ef";
        // SHA256 of "mypasswd"
        let expected = "0316001ef027cb1e25658d9faa50cb4c685223867f8a4d42b7994d817f0d2424";
        let expected_ga = GenericArray::clone_from_slice(&hex::decode(expected).unwrap());

        std::env::set_var("AAAABBBBCCCC", "mypasswd");

        let key = Config::init_key(eks);

        assert_eq!(expected_ga, key);
        assert_eq!(std::env::var("AAAABBBBCCCC").unwrap(), "");

        std::env::remove_var("AAAABBBBCCCC");
    }
}
