use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::config::ConfigError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub last_check_timestamp: i64,
    pub faction_id: Option<i64>,
}

impl State {
    pub fn load_or_create(path: &Path) -> Result<Self> {
        if path.exists() {
            Self::load(path)
        } else {
            Ok(Self {
                last_check_timestamp: 0,
                faction_id: None,
            })
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path).map_err(|e| StateError::Io(e.to_string()))?;

        serde_json::from_str(&contents).map_err(|e| StateError::Parse(e.to_string()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| StateError::Io(e.to_string()))?;
        }

        let contents =
            serde_json::to_string_pretty(self).map_err(|e| StateError::Serialize(e.to_string()))?;

        fs::write(path, contents).map_err(|e| StateError::Io(e.to_string()))?;

        Ok(())
    }

    pub fn update_timestamp(&mut self, timestamp: i64) {
        self.last_check_timestamp = timestamp;
    }
}

#[derive(Debug, Error)]
pub enum StateError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Failed to parse state file: {0}")]
    Parse(String),

    #[error("Failed to serialize state: {0}")]
    Serialize(String),
}

impl From<StateError> for ConfigError {
    fn from(_: StateError) -> Self {
        ConfigError::StateError
    }
}

pub type Result<T> = std::result::Result<T, StateError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn test_state_path(tmp_dir: &TempDir) -> PathBuf {
        tmp_dir.path().join("state.json")
    }

    #[test]
    fn test_state_default_timestamp() {
        let tmp_dir = TempDir::new().unwrap();
        let state_path = test_state_path(&tmp_dir);

        let state = State::load_or_create(&state_path).unwrap();
        assert_eq!(state.last_check_timestamp, 0);
    }

    #[test]
    fn test_state_save_and_load() {
        let tmp_dir = TempDir::new().unwrap();
        let state_path = test_state_path(&tmp_dir);

        let state1 = State {
            last_check_timestamp: 1234567890,
            faction_id: Some(12345),
        };
        state1.save(&state_path).unwrap();

        let state2 = State::load(&state_path).unwrap();
        assert_eq!(state2.last_check_timestamp, 1234567890);
        assert_eq!(state2.faction_id, Some(12345));
    }

    #[test]
    fn test_state_load_nonexistent() {
        let tmp_dir = TempDir::new().unwrap();
        let state_path = tmp_dir.path().join("nonexistent.json");

        let state = State::load_or_create(&state_path).unwrap();
        assert_eq!(state.last_check_timestamp, 0);
    }

    #[test]
    fn test_state_update_timestamp() {
        let tmp_dir = TempDir::new().unwrap();
        let state_path = test_state_path(&tmp_dir);

        let mut state = State::load_or_create(&state_path).unwrap();
        state.update_timestamp(9999999999);
        state.save(&state_path).unwrap();

        let loaded = State::load(&state_path).unwrap();
        assert_eq!(loaded.last_check_timestamp, 9999999999);
    }

    #[test]
    fn test_state_serialization() {
        let state = State {
            last_check_timestamp: 1234567890,
            faction_id: Some(54321),
        };

        let json = serde_json::to_string_pretty(&state).unwrap();
        let loaded: State = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.last_check_timestamp, state.last_check_timestamp);
        assert_eq!(loaded.faction_id, state.faction_id);
    }

    #[test]
    fn test_state_serialization_without_faction_id() {
        let state = State {
            last_check_timestamp: 999,
            faction_id: None,
        };

        let json = serde_json::to_string_pretty(&state).unwrap();
        let loaded: State = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.last_check_timestamp, 999);
        assert_eq!(loaded.faction_id, None);
    }
}
