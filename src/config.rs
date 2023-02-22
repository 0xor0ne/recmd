use sha2::{
    digest::crypto_common::generic_array::{typenum::U32, GenericArray},
    Digest, Sha256,
};
use std::time::Duration;

const TCP_CONNECT_TO: Duration = Duration::new(5, 0);
const TCP_RESP_TO: Duration = Duration::new(5, 0);
const TCP_WRITE_TO: Duration = Duration::new(10, 0);
const HISTORY_DEPTH: usize = 100_000;
const PASSWORD_DEF: &str = "1e$tob5UtRi6oFr8jlYO";

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
        let mut hasher = Sha256::new();
        hasher.update(PASSWORD_DEF.as_bytes());
        let key = hasher.finalize();

        Config {
            key,
            history_depth: HISTORY_DEPTH,
            tcp_connect_to: TCP_CONNECT_TO,
            tcp_write_to: TCP_WRITE_TO,
            tcp_resp_to: TCP_RESP_TO,
        }
    }

    pub fn get_key(&self) -> GenericArray<u8, U32> {
        self.key.clone()
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
    #[test]
    fn init_and_check_len() {
        let c = Config::init();
        assert_eq!(c.get_key().len(), 32);
    }
}
