use std::env;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub faction_id: Option<i64>,
    pub state_file: PathBuf,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub api_key: Option<String>,
    pub faction_id: Option<i64>,
    pub state_path: Option<PathBuf>,
}

impl Config {
    pub fn from_args(
        api_key: Option<String>,
        faction_id: Option<i64>,
        state_path: Option<PathBuf>,
    ) -> Result<Self> {
        let api_key = Self::resolve_api_key(api_key)?;
        let state_file = Self::resolve_state_path(state_path);

        Ok(Config {
            api_key,
            faction_id,
            state_file,
        })
    }

    fn resolve_api_key(provided: Option<String>) -> Result<String> {
        if let Some(key) = provided
            && !key.is_empty()
        {
            return Ok(key);
        }

        if let Ok(key) = env::var("TORN_API_KEY")
            && !key.is_empty()
        {
            return Ok(key);
        }

        if let Ok(key) = env::var("TORN_KEY")
            && !key.is_empty()
        {
            return Ok(key);
        }

        if let Ok(cwd) = env::current_dir() {
            let env_path = cwd.join(".env");
            if env_path.exists()
                && let Ok(contents) = fs::read_to_string(&env_path)
            {
                for line in contents.lines() {
                    let line = line.trim();
                    if line.starts_with('#') || line.is_empty() {
                        continue;
                    }
                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim();
                        let value = value.trim();
                        if (key == "TORN_API_KEY" || key == "TORN_KEY") && !value.is_empty() {
                            return Ok(value.to_string());
                        }
                    }
                }
            }
        }

        Err(ConfigError::ApiKeyNotFound)
    }

    fn resolve_state_path(provided: Option<PathBuf>) -> PathBuf {
        if let Some(path) = provided {
            return path;
        }

        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("faction-hits");
            return app_dir.join("state.json");
        }

        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".faction-hits-state.json")
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(
        "API key not found. Set TORN_API_KEY environment variable, .env file, or use --api-key"
    )]
    ApiKeyNotFound,

    #[error("Failed to load or save state")]
    StateError,
}

pub type Result<T> = std::result::Result<T, ConfigError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_api_key_from_argument() {
        let config = Config::from_args(Some("test-key-from-arg".to_string()), None, None);
        assert!(config.is_ok());
        assert_eq!(config.unwrap().api_key, "test-key-from-arg");
    }

    #[test]
    fn test_resolve_state_path_default() {
        let config = Config::from_args(Some("test-key".to_string()), None, None).unwrap();
        assert!(config.state_file.to_str().unwrap().contains("faction-hits"));
    }

    #[test]
    fn test_resolve_state_path_custom() {
        let tmp_dir = TempDir::new().unwrap();
        let custom_path = tmp_dir.path().join("custom-state.json");

        let config = Config::from_args(
            Some("test-key".to_string()),
            None,
            Some(custom_path.clone()),
        )
        .unwrap();

        assert_eq!(config.state_file, custom_path);
    }

    #[test]
    fn test_config_error_display() {
        let error = ConfigError::ApiKeyNotFound;
        let display = format!("{}", error);
        assert!(display.contains("API key") || display.contains("TORN_API_KEY"));
    }

    #[test]
    fn test_resolve_api_key_from_env_var() {
        unsafe {
            env::set_var("TORN_API_KEY", "env-key-test");
        }
        let config = Config::from_args(None, None, None);
        unsafe {
            env::remove_var("TORN_API_KEY");
        }
        assert!(config.is_ok());
        assert_eq!(config.unwrap().api_key, "env-key-test");
    }

    #[test]
    fn test_resolve_api_key_from_torn_key_env() {
        unsafe {
            env::set_var("TORN_KEY", "torn-key-test");
        }
        let config = Config::from_args(None, None, None);
        unsafe {
            env::remove_var("TORN_KEY");
        }
        assert!(config.is_ok());
        assert_eq!(config.unwrap().api_key, "torn-key-test");
    }

    #[test]
    fn test_resolve_api_key_empty_arg_falls_through() {
        unsafe {
            env::set_var("TORN_API_KEY", "env-fallback-key");
        }
        let config = Config::from_args(Some("".to_string()), None, None);
        unsafe {
            env::remove_var("TORN_API_KEY");
        }
        assert!(config.is_ok());
        assert_eq!(config.unwrap().api_key, "env-fallback-key");
    }

    #[test]
    fn test_resolve_api_key_no_key_available() {
        unsafe {
            env::remove_var("TORN_API_KEY");
        }
        unsafe {
            env::remove_var("TORN_KEY");
        }
        let tmp_dir = TempDir::new().unwrap();
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(tmp_dir.path()).unwrap();

        let config = Config::from_args(None, None, None);
        env::set_current_dir(original_dir).unwrap();

        assert!(config.is_err());
        matches!(config.unwrap_err(), ConfigError::ApiKeyNotFound);
    }

    #[test]
    fn test_faction_id_passed_through() {
        let config = Config::from_args(Some("key".to_string()), Some(12345), None).unwrap();
        assert_eq!(config.faction_id, Some(12345));
    }

    #[test]
    fn test_faction_id_none() {
        let config = Config::from_args(Some("key".to_string()), None, None).unwrap();
        assert_eq!(config.faction_id, None);
    }
}
