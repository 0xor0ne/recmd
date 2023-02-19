use sha2::{
    digest::crypto_common::generic_array::{typenum::U32, GenericArray},
    Digest, Sha256,
};

const HISTORY_DEPTH: usize = 100_000;
const PASSWORD_DEF: &str = "1e$tob5UtRi6oFr8jlYO";

pub struct Config {
    key: GenericArray<u8, U32>,
}

impl Config {
    pub fn init() -> Self {
        let mut hasher = Sha256::new();
        hasher.update(PASSWORD_DEF.as_bytes());
        let key = hasher.finalize();

        Config { key }
    }

    pub fn get_key(&self) -> GenericArray<u8, U32> {
        self.key.clone()
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
