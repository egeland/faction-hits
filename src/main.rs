use anyhow::Result;
use chrono::DateTime;
use clap::Parser;
use std::path::PathBuf;

mod api;
mod config;
mod hits;
mod state;

use api::TornClient;
use config::{AppConfig, Config};
use state::State;

#[derive(Parser, Debug)]
#[command(name = "faction-hits")]
#[command(about = "Track non-anonymous hits on faction members", long_about = None)]
struct Args {
    #[arg(short, long, help = "Torn API key")]
    #[arg(env = "TORN_API_KEY")]
    api_key: Option<String>,

    #[arg(
        short,
        long,
        help = "Faction ID (optional, defaults to key owner's faction)"
    )]
    faction_id: Option<i64>,

    #[arg(short, long, help = "Path to state file")]
    state_file: Option<PathBuf>,
}

impl From<Args> for AppConfig {
    fn from(args: Args) -> Self {
        Self {
            api_key: args.api_key,
            faction_id: args.faction_id,
            state_path: args.state_file,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let app_config: AppConfig = args.into();

    let config = Config::from_args(
        app_config.api_key,
        app_config.faction_id,
        app_config.state_path,
    )?;

    let state = State::load_or_create(&config.state_file)?;

    let client = TornClient::new(&config.api_key);

    println!(
        "Fetching faction attacks since timestamp {}...",
        state.last_check_timestamp
    );

    let attacks = client
        .get_faction_attacks(config.faction_id, Some(state.last_check_timestamp))
        .await?;

    println!("Found {} total attacks", attacks.len());

    let new_hits = hits::filter_new_hits(&attacks, &state);

    if new_hits.is_empty() {
        println!("No new non-anonymous hits found.");
    } else {
        println!("\n=== {} New Non-Anonymous Hits ===\n", new_hits.len());
        for (i, hit) in new_hits.iter().enumerate() {
            let datetime = DateTime::from_timestamp(hit.timestamp, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            println!(
                "{}. {} ({}) attacked {} ({})",
                i + 1,
                hit.attacker_name,
                hit.attacker_id,
                hit.defender_name,
                hit.defender_id
            );
            println!(
                "   Result: {} | Respect: {:.2} | Time: {}",
                hit.result, hit.respect, datetime
            );
            println!();
        }
    }

    if let Some(latest) = hits::get_latest_timestamp(&attacks) {
        let mut new_state = state;
        new_state.update_timestamp(latest);
        new_state.save(&config.state_file)?;
        println!("State updated. Last check timestamp: {}", latest);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_to_app_config() {
        let args = Args {
            api_key: Some("test-key".to_string()),
            faction_id: Some(12345),
            state_file: Some(PathBuf::from("/tmp/state.json")),
        };

        let config: AppConfig = args.into();
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.faction_id, Some(12345));
        assert_eq!(config.state_path, Some(PathBuf::from("/tmp/state.json")));
    }

    #[test]
    fn test_args_defaults() {
        let args = Args {
            api_key: None,
            faction_id: None,
            state_file: None,
        };

        let config: AppConfig = args.into();
        assert_eq!(config.api_key, None);
        assert_eq!(config.faction_id, None);
        assert_eq!(config.state_path, None);
    }
}
