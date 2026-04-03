use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionAttack {
    #[serde(default)]
    pub id: i64,
    pub attacker_id: i64,
    pub attacker_name: String,
    pub defender_id: i64,
    pub defender_name: String,
    pub result: String,
    #[serde(rename = "stealthed")]
    pub stealth: bool,
    pub respect: f64,
    #[serde(rename = "timestamp_ended")]
    pub timestamp: i64,
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetail {
    code: u32,
    #[serde(default)]
    error: String,
}

pub struct TornClient {
    api_key: String,
    client: reqwest::Client,
}

impl TornClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_faction_attacks(
        &self,
        faction_id: Option<i64>,
        from_timestamp: Option<i64>,
    ) -> Result<Vec<FactionAttack>> {
        let url = if let Some(id) = faction_id {
            format!(
                "https://api.torn.com/faction/{}?selections=attacksfull&key={}",
                id, self.api_key
            )
        } else {
            format!(
                "https://api.torn.com/faction/?selections=attacksfull&key={}",
                self.api_key
            )
        };

        let mut request = self.client.get(&url);

        if let Some(from) = from_timestamp {
            request = request.query(&[("from", from.to_string())]);
        }

        let response = request
            .send()
            .await
            .map_err(|e| TornApiError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TornApiError::Api(format!("{}: {}", status, body)));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| TornApiError::Parse(e.to_string()))?;

        if let Some(errors) = data.get("error") {
            if let Ok(err_detail) = serde_json::from_value::<ApiErrorDetail>(errors.clone()) {
                let err = TornApiError::from_api_error(err_detail.code, &err_detail.error);
                return Err(err);
            }
            return Err(TornApiError::Api(format!("Torn API error: {:?}", errors)));
        }

        let attacks_map = data
            .get("attacksfull")
            .and_then(|a| a.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(id, value)| {
                        let mut attack: FactionAttack =
                            serde_json::from_value(value.clone()).ok()?;
                        attack.id = id.parse().ok()?;
                        Some(attack)
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(attacks_map)
    }
}

#[derive(Debug, Error)]
pub enum TornApiError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Failed to parse response: {0}")]
    Parse(String),

    #[error("{context}")]
    PermissionDenied { context: String, code: u32 },
}

impl TornApiError {
    fn from_api_error(code: u32, message: &str) -> Self {
        match code {
            7 => TornApiError::PermissionDenied {
                context: "Your API key doesn't have permission to access faction attacks. \
                    Please regenerate your key at https://www.torn.com/preferences.php#tab=api \
                    and ensure you select 'attacks' or 'attacksfull' under the Faction section."
                    .to_string(),
                code,
            },
            2 => TornApiError::Api(format!("Invalid API key: {}", message)),
            16 => TornApiError::PermissionDenied {
                context: "Your API key's access level is too low for this operation. \
                    Please regenerate your key with higher permissions at \
                    https://www.torn.com/preferences.php#tab=api"
                    .to_string(),
                code,
            },
            _ => TornApiError::Api(format!("Torn API error (code {}): {}", code, message)),
        }
    }
}

pub type Result<T> = std::result::Result<T, TornApiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faction_attack_deserialization() {
        let json = serde_json::json!({
            "attacker_id": 111,
            "attacker_name": "Attacker",
            "defender_id": 222,
            "defender_name": "Defender",
            "result": "Lost",
            "stealthed": false,
            "respect": 1.5,
            "timestamp_ended": 1234567890
        });

        let attack: FactionAttack = serde_json::from_value(json).unwrap();
        assert_eq!(attack.attacker_id, 111);
        assert_eq!(attack.defender_name, "Defender");
        assert!(!attack.stealth);
        assert_eq!(attack.respect, 1.5);
    }

    #[test]
    fn test_faction_attack_stealth_true() {
        let json = serde_json::json!({
            "attacker_id": 333,
            "attacker_name": "Stealthy",
            "defender_id": 444,
            "defender_name": "Target",
            "result": "Attacked",
            "stealthed": true,
            "respect": 0.5,
            "timestamp_ended": 1234567891
        });

        let attack: FactionAttack = serde_json::from_value(json).unwrap();
        assert!(attack.stealth);
    }

    #[test]
    fn test_faction_attack_various_results() {
        let results = [
            "Lost",
            "Attacked",
            "Hospitalized",
            "Mugged",
            "Lost",
            "Attacked",
        ];
        for result in results {
            let json = serde_json::json!({
                "attacker_id": 1,
                "attacker_name": "Test",
                "defender_id": 2,
                "defender_name": "Target",
                "result": result,
                "stealthed": false,
                "respect": 1.0,
                "timestamp_ended": 1234567890
            });

            let attack: FactionAttack = serde_json::from_value(json).unwrap();
            assert_eq!(attack.result, result);
        }
    }

    #[test]
    fn test_client_creation() {
        let client = TornClient::new("test-api-key");
        assert_eq!(client.api_key, "test-api-key");
    }

    #[test]
    fn test_api_error_permission_denied() {
        let err = TornApiError::from_api_error(7, "Incorrect ID-entity relation");
        match err {
            TornApiError::PermissionDenied { context, code } => {
                assert_eq!(code, 7);
                assert!(context.contains("attacks"));
                assert!(context.contains("torn.com/preferences.php"));
            }
            _ => panic!("Expected PermissionDenied variant"),
        }
    }

    #[test]
    fn test_api_error_access_level_too_low() {
        let err = TornApiError::from_api_error(16, "Access level too low");
        match err {
            TornApiError::PermissionDenied { context, code } => {
                assert_eq!(code, 16);
                assert!(context.contains("access level"));
            }
            _ => panic!("Expected PermissionDenied variant"),
        }
    }

    #[test]
    fn test_api_error_invalid_key() {
        let err = TornApiError::from_api_error(2, "Incorrect Key");
        match err {
            TornApiError::Api(msg) => {
                assert!(msg.contains("Invalid API key"));
                assert!(msg.contains("Incorrect Key"));
            }
            _ => panic!("Expected Api variant"),
        }
    }

    #[test]
    fn test_api_error_unknown_code() {
        let err = TornApiError::from_api_error(99, "Some unknown error");
        match err {
            TornApiError::Api(msg) => {
                assert!(msg.contains("code 99"));
                assert!(msg.contains("Some unknown error"));
            }
            _ => panic!("Expected Api variant"),
        }
    }

    #[test]
    fn test_api_error_detail_deserialization() {
        let json = serde_json::json!({
            "code": 7,
            "error": "Incorrect ID-entity relation"
        });

        let err_detail: ApiErrorDetail = serde_json::from_value(json).unwrap();
        assert_eq!(err_detail.code, 7);
        assert_eq!(err_detail.error, "Incorrect ID-entity relation");
    }
}
