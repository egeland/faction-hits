#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time;

use crate::discord::Storage;
use crate::TornClient;
use faction_hits::FactionAttack;

pub struct Scheduler {
    storage: Arc<RwLock<Storage>>,
    default_api_key: String,
    check_interval: Duration,
}

impl Scheduler {
    pub fn new(
        storage: Arc<RwLock<Storage>>,
        default_api_key: String,
        check_interval_secs: u64,
    ) -> Self {
        Self {
            storage,
            default_api_key,
            check_interval: Duration::from_secs(check_interval_secs),
        }
    }

    pub async fn check_all_guilds<F, Fut>(&self, mut callback: F)
    where
        F: FnMut(u64, u64, i64, String, Vec<FactionAttack>) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let storage = self.storage.read().await;
        let guilds: Vec<_> = storage
            .guilds
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect();
        drop(storage);

        for (guild_id, config) in guilds {
            let api_key = config.api_key.as_deref().unwrap_or(&self.default_api_key);

            let client = TornClient::new(api_key);
            match client
                .get_faction_attacks(Some(config.faction_id), Some(config.last_check_timestamp))
                .await
            {
                Ok(attacks) => {
                    let new_hits: Vec<_> = attacks
                        .iter()
                        .filter(|a| a.timestamp > config.last_check_timestamp && !a.stealth)
                        .cloned()
                        .collect();

                    if !new_hits.is_empty() {
                        callback(
                            guild_id,
                            config.channel_id,
                            config.faction_id,
                            api_key.to_string(),
                            new_hits.clone(),
                        )
                        .await;

                        if let Some(latest) = new_hits.iter().map(|a| a.timestamp).max() {
                            let mut storage = self.storage.write().await;
                            storage.update_timestamp(guild_id, latest);
                            // Persist storage after updating timestamp
                            if let Err(e) = storage.save(
                                &dirs::config_dir()
                                    .map(|p| p.join("faction-hits").join("discord-storage.json"))
                                    .unwrap_or_else(|| PathBuf::from("discord-storage.json")),
                            ) {
                                tracing::warn!("Failed to persist storage: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch attacks for guild {}: {}", guild_id, e);
                }
            }
        }
    }

    pub async fn run<F, Fut>(&self, mut callback: F)
    where
        F: FnMut(u64, u64, i64, String, Vec<FactionAttack>) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let mut interval = time::interval(self.check_interval);

        loop {
            interval.tick().await;
            self.check_all_guilds(&mut callback).await;
        }
    }
}

pub fn format_hits_message(hits: &[FactionAttack]) -> String {
    let filtered: Vec<_> = hits.iter().filter(|h| !h.stealth).collect();

    if filtered.is_empty() {
        return "No new non-anonymous hits found.".to_string();
    }

    let hit_label = if filtered.len() == 1 { "Hit" } else { "Hits" };
    let mut message = format!(
        "=== {} New Non-Anonymous {} ===\n\n",
        filtered.len(),
        hit_label
    );

    for (i, hit) in filtered.iter().enumerate() {
        let datetime = chrono::DateTime::from_timestamp(hit.timestamp, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        message.push_str(&format!(
            "{}. **{}** ({}) attacked **{}** ({})\n",
            i + 1,
            hit.attacker_name,
            hit.attacker_id,
            hit.defender_name,
            hit.defender_id
        ));
        message.push_str(&format!(
            "   Result: {} | Respect: {:.2} | Time: {}\n\n",
            hit.result, hit.respect, datetime
        ));
    }

    message
}

#[cfg(test)]
mod tests {
    use super::*;
    use faction_hits::FactionAttack;
    use tokio::sync::RwLock;

    fn create_test_attacks() -> Vec<FactionAttack> {
        vec![
            FactionAttack {
                id: 1,
                attacker_id: 111,
                attacker_name: "Attacker1".to_string(),
                defender_id: 222,
                defender_name: "Defender1".to_string(),
                result: "Lost".to_string(),
                stealth: false,
                respect: 1.5,
                timestamp: 100,
            },
            FactionAttack {
                id: 2,
                attacker_id: 333,
                attacker_name: "Stealthy".to_string(),
                defender_id: 444,
                defender_name: "Defender2".to_string(),
                result: "Attacked".to_string(),
                stealth: true,
                respect: 0.5,
                timestamp: 101,
            },
            FactionAttack {
                id: 3,
                attacker_id: 555,
                attacker_name: "Attacker2".to_string(),
                defender_id: 666,
                defender_name: "Defender3".to_string(),
                result: "Hospitalized".to_string(),
                stealth: false,
                respect: 2.25,
                timestamp: 102,
            },
        ]
    }

    #[test]
    fn test_format_hits_message_empty() {
        let message = format_hits_message(&[]);
        assert_eq!(message, "No new non-anonymous hits found.");
    }

    #[test]
    fn test_format_hits_message_single() {
        let attacks = vec![create_test_attacks()[0].clone()];
        let message = format_hits_message(&attacks);
        assert!(message.contains("1 New Non-Anonymous Hit"));
        assert!(message.contains("Attacker1"));
        assert!(message.contains("Defender1"));
    }

    #[test]
    fn test_format_hits_message_filters_stealth() {
        let attacks = create_test_attacks();
        let message = format_hits_message(&attacks);
        assert!(message.contains("2 New Non-Anonymous Hits"));
        assert!(!message.contains("Stealthy"));
    }

    #[tokio::test]
    async fn test_scheduler_new() {
        let storage = Arc::new(RwLock::new(Storage::default()));
        let scheduler = Scheduler::new(storage, "test-key".to_string(), 300);
        assert_eq!(scheduler.check_interval.as_secs(), 300);
    }

    #[tokio::test]
    async fn test_scheduler_with_custom_interval() {
        let storage = Arc::new(RwLock::new(Storage::default()));
        let scheduler = Scheduler::new(storage, "key".to_string(), 600);
        assert_eq!(scheduler.check_interval.as_secs(), 600);
    }
}
