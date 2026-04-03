#![allow(dead_code)]

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::discord::Storage;

pub struct Bot {
    pub storage: Arc<RwLock<Storage>>,
    pub default_api_key: String,
    pub check_interval_secs: u64,
}

impl Bot {
    pub fn new(default_api_key: String) -> Self {
        Self {
            storage: Arc::new(RwLock::new(Storage::default())),
            default_api_key,
            check_interval_secs: 300,
        }
    }

    pub fn with_storage(self, storage: Storage) -> Self {
        Self {
            storage: Arc::new(RwLock::new(storage)),
            ..self
        }
    }

    pub fn with_check_interval(mut self, interval_secs: u64) -> Self {
        self.check_interval_secs = interval_secs;
        self
    }

    pub fn get_api_key(&self, guild_key: Option<&str>) -> String {
        guild_key
            .map(|k| k.to_string())
            .unwrap_or_else(|| self.default_api_key.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bot_default_creation() {
        let bot = Bot::new("default-key".to_string());
        assert_eq!(bot.default_api_key, "default-key");
        assert_eq!(bot.check_interval_secs, 300);
    }

    #[test]
    fn test_bot_with_storage() {
        let storage = Storage::default();
        let bot = Bot::new("key".to_string()).with_storage(storage.clone());
        assert!(bot.storage.try_read().is_ok());
    }

    #[test]
    fn test_bot_with_check_interval() {
        let bot = Bot::new("key".to_string()).with_check_interval(600);
        assert_eq!(bot.check_interval_secs, 600);
    }

    #[test]
    fn test_get_api_key_guild_specific() {
        let bot = Bot::new("default".to_string());
        let key = bot.get_api_key(Some("guild-key"));
        assert_eq!(key, "guild-key");
    }

    #[test]
    fn test_get_api_key_default() {
        let bot = Bot::new("default".to_string());
        let key = bot.get_api_key(None);
        assert_eq!(key, "default");
    }

    #[test]
    fn test_bot_all_options() {
        let storage = Storage::default();
        let bot = Bot::new("key".to_string())
            .with_storage(storage)
            .with_check_interval(120);

        assert_eq!(bot.default_api_key, "key");
        assert_eq!(bot.check_interval_secs, 120);
    }
}
