#![allow(dead_code)]

use serenity::model::application::CommandInteraction;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandType {
    Setup,
    SetupApiKey,
    Status,
    Hits,
    Help,
}

impl CommandType {
    pub fn from_str(name: &str) -> Option<Self> {
        match name {
            "setup" => Some(Self::Setup),
            "setup-api-key" => Some(Self::SetupApiKey),
            "status" => Some(Self::Status),
            "hits" => Some(Self::Hits),
            "help" => Some(Self::Help),
            _ => None,
        }
    }
}

pub fn parse_setup_command(command: &CommandInteraction) -> Result<(u64, i64), CommandParseError> {
    let channel_id = command
        .data
        .options
        .iter()
        .find(|o| o.name == "channel")
        .and_then(|o| o.value.as_channel_id())
        .map(|c| c.get())
        .ok_or(CommandParseError::MissingArgument("channel"))?;

    let faction_id = command
        .data
        .options
        .iter()
        .find(|o| o.name == "faction-id")
        .and_then(|o| o.value.as_i64())
        .ok_or(CommandParseError::MissingArgument("faction-id"))?;

    Ok((channel_id, faction_id))
}

pub fn parse_api_key_command(command: &CommandInteraction) -> Result<String, CommandParseError> {
    command
        .data
        .options
        .iter()
        .find(|o| o.name == "api-key")
        .and_then(|o| o.value.as_str())
        .map(|s| s.to_string())
        .ok_or(CommandParseError::MissingArgument("api-key"))
}

#[derive(Debug, Error)]
pub enum CommandParseError {
    #[error("Missing required argument: {0}")]
    MissingArgument(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_type_from_str_valid() {
        assert_eq!(CommandType::from_str("setup"), Some(CommandType::Setup));
        assert_eq!(
            CommandType::from_str("setup-api-key"),
            Some(CommandType::SetupApiKey)
        );
        assert_eq!(CommandType::from_str("status"), Some(CommandType::Status));
        assert_eq!(CommandType::from_str("hits"), Some(CommandType::Hits));
        assert_eq!(CommandType::from_str("help"), Some(CommandType::Help));
    }

    #[test]
    fn test_command_type_from_str_invalid() {
        assert_eq!(CommandType::from_str("unknown"), None);
        assert_eq!(CommandType::from_str(""), None);
        assert_eq!(CommandType::from_str("setup "), None);
    }

    #[test]
    fn test_command_parse_error_display() {
        let err = CommandParseError::MissingArgument("test-arg");
        assert!(err.to_string().contains("test-arg"));
    }
}
