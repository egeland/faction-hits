#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildConfig {
    pub channel_id: u64,
    pub faction_id: i64,
    pub last_check_timestamp: i64,
    pub api_key: Option<String>,
}

impl GuildConfig {
    pub fn new(channel_id: u64, faction_id: i64) -> Self {
        Self {
            channel_id,
            faction_id,
            last_check_timestamp: 0,
            api_key: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Storage {
    pub guilds: HashMap<u64, GuildConfig>,
}

impl Storage {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = fs::read_to_string(path).map_err(|e| StorageError::Io(e.to_string()))?;
        if contents.trim().is_empty() {
            return Ok(Self::default());
        }
        let result: std::result::Result<Storage, _> = serde_json::from_str(&contents);
        match result {
            Ok(storage) => Ok(storage),
            Err(_) => Ok(Self::default()),
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| StorageError::Io(e.to_string()))?;
        }
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| StorageError::Serialize(e.to_string()))?;
        fs::write(path, contents).map_err(|e| StorageError::Io(e.to_string()))?;
        Ok(())
    }

    pub fn add_guild(&mut self, guild_id: u64, config: GuildConfig) {
        self.guilds.insert(guild_id, config);
    }

    pub fn remove_guild(&mut self, guild_id: u64) -> Option<GuildConfig> {
        self.guilds.remove(&guild_id)
    }

    pub fn get_guild(&self, guild_id: u64) -> Option<&GuildConfig> {
        self.guilds.get(&guild_id)
    }

    pub fn get_guild_mut(&mut self, guild_id: u64) -> Option<&mut GuildConfig> {
        self.guilds.get_mut(&guild_id)
    }

    pub fn update_timestamp(&mut self, guild_id: u64, timestamp: i64) -> bool {
        if let Some(config) = self.guilds.get_mut(&guild_id) {
            config.last_check_timestamp = timestamp;
            true
        } else {
            false
        }
    }

    pub fn update_api_key(&mut self, guild_id: u64, api_key: String) -> bool {
        if let Some(config) = self.guilds.get_mut(&guild_id) {
            config.api_key = Some(api_key);
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Failed to parse storage file: {0}")]
    Parse(String),
    #[error("Failed to serialize storage: {0}")]
    Serialize(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn test_storage_path(tmp_dir: &TempDir) -> PathBuf {
        tmp_dir.path().join("storage.json")
    }

    #[test]
    fn test_storage_default() {
        let storage = Storage::default();
        assert!(storage.guilds.is_empty());
    }

    #[test]
    fn test_storage_empty_file() {
        let tmp_dir = TempDir::new().unwrap();
        let path = test_storage_path(&tmp_dir);
        fs::write(&path, "{}").unwrap();
        let storage = Storage::load(&path).unwrap();
        assert!(storage.guilds.is_empty());
    }

    #[test]
    fn test_storage_nonexistent_file() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("nonexistent.json");
        let storage = Storage::load(&path).unwrap();
        assert!(storage.guilds.is_empty());
    }

    #[test]
    fn test_guild_config_new() {
        let config = GuildConfig::new(123, 456);
        assert_eq!(config.channel_id, 123);
        assert_eq!(config.faction_id, 456);
        assert_eq!(config.last_check_timestamp, 0);
        assert_eq!(config.api_key, None);
    }

    #[test]
    fn test_add_guild() {
        let mut storage = Storage::default();
        storage.add_guild(123, GuildConfig::new(456, 789));
        assert_eq!(storage.guilds.len(), 1);
        assert!(storage.get_guild(123).is_some());
    }

    #[test]
    fn test_remove_guild() {
        let mut storage = Storage::default();
        storage.add_guild(123, GuildConfig::new(456, 789));
        let removed = storage.remove_guild(123);
        assert!(removed.is_some());
        assert!(storage.get_guild(123).is_none());
    }

    #[test]
    fn test_get_guild_none() {
        let storage = Storage::default();
        assert!(storage.get_guild(999).is_none());
    }

    #[test]
    fn test_update_timestamp() {
        let mut storage = Storage::default();
        storage.add_guild(123, GuildConfig::new(456, 789));
        let result = storage.update_timestamp(123, 999);
        assert!(result);
        assert_eq!(storage.get_guild(123).unwrap().last_check_timestamp, 999);
    }

    #[test]
    fn test_update_timestamp_nonexistent() {
        let mut storage = Storage::default();
        let result = storage.update_timestamp(999, 999);
        assert!(!result);
    }

    #[test]
    fn test_update_api_key() {
        let mut storage = Storage::default();
        storage.add_guild(123, GuildConfig::new(456, 789));
        let result = storage.update_api_key(123, "test-key".to_string());
        assert!(result);
        assert_eq!(
            storage.get_guild(123).unwrap().api_key,
            Some("test-key".to_string())
        );
    }

    #[test]
    fn test_save_and_load() {
        let tmp_dir = TempDir::new().unwrap();
        let path = test_storage_path(&tmp_dir);

        let mut storage = Storage::default();
        storage.add_guild(1, GuildConfig::new(10, 100));
        storage.add_guild(2, GuildConfig::new(20, 200));

        storage.save(&path).unwrap();

        let loaded = Storage::load(&path).unwrap();
        assert_eq!(loaded.guilds.len(), 2);
        assert_eq!(loaded.get_guild(1).unwrap().faction_id, 100);
        assert_eq!(loaded.get_guild(2).unwrap().faction_id, 200);
    }

    #[test]
    fn test_get_guild_mut() {
        let mut storage = Storage::default();
        storage.add_guild(123, GuildConfig::new(456, 789));

        let config = storage.get_guild_mut(123).unwrap();
        config.last_check_timestamp = 555;

        assert_eq!(storage.get_guild(123).unwrap().last_check_timestamp, 555);
    }
}
