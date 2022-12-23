use std::sync::*;

pub struct Config {
    pub server_addr: String,
    pub data_dir: String,
    pub orm_addr: String,
}

impl Config {
    fn default() -> Config {
        Config {
            server_addr: "0.0.0.0:3000".to_string(),
            data_dir: "/data/oct".to_string(),
            orm_addr: "127.0.0.1:8000".to_string(),
        }
    }
}

lazy_static! {
    static ref CONFIG: RwLock<Config> = RwLock::new(Config::default());
}

pub fn config() -> RwLockReadGuard<'static, Config> {
    CONFIG.read().unwrap()
}

pub fn config_write() -> RwLockWriteGuard<'static, Config> {
    CONFIG.write().unwrap()
}
