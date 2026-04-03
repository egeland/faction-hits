mod discord;

use anyhow::{anyhow, Result};
use chrono::DateTime;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use faction_hits::{filter_new_hits, get_latest_timestamp, AppConfig, Config, State, TornClient};

struct BotEventHandler;

#[serenity::async_trait]
impl serenity::client::EventHandler for BotEventHandler {
    async fn ready(
        &self,
        _ctx: serenity::client::Context,
        _ready: serenity::model::gateway::Ready,
    ) {
        tracing::info!("Bot is ready!");
    }
}

#[derive(Parser)]
#[command(name = "faction-hits")]
#[command(about = "Track non-anonymous hits on faction members", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Run as a Discord bot")]
    Bot {
        #[arg(long, help = "Discord bot token")]
        #[arg(env = "DISCORD_BOT_TOKEN")]
        discord_token: Option<String>,

        #[arg(long, help = "Check interval in seconds")]
        check_interval: Option<u64>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Commands::Bot {
            discord_token,
            check_interval,
        }) => {
            run_bot(discord_token, check_interval).await?;
        }
        None => {
            run_cli().await?;
        }
    }

    Ok(())
}

async fn run_cli() -> Result<()> {
    let args = cli_args()?;
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

    let new_hits = filter_new_hits(&attacks, &state);

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
                " Result: {} | Respect: {:.2} | Time: {}",
                hit.result, hit.respect, datetime
            );
            println!();
        }
    }

    if let Some(latest) = get_latest_timestamp(&attacks) {
        let mut new_state = state;
        new_state.update_timestamp(latest);
        new_state.save(&config.state_file)?;
        println!("State updated. Last check timestamp: {}", latest);
    }

    Ok(())
}

fn cli_args() -> Result<CliArgs> {
    let api_key = std::env::var("TORN_API_KEY").ok().filter(|k| !k.is_empty());
    let args: CliArgs = ClapArgs::parse().into();
    Ok(CliArgs {
        api_key,
        faction_id: args.faction_id,
        state_file: args.state_file,
    })
}

struct CliArgs {
    api_key: Option<String>,
    faction_id: Option<i64>,
    state_file: Option<PathBuf>,
}

#[derive(Parser)]
struct ClapArgs {
    #[arg(
        short,
        long,
        help = "Faction ID (optional, defaults to key owner's faction)"
    )]
    faction_id: Option<i64>,

    #[arg(short, long, help = "Path to state file")]
    state_file: Option<PathBuf>,
}

impl From<ClapArgs> for CliArgs {
    fn from(args: ClapArgs) -> Self {
        Self {
            api_key: None,
            faction_id: args.faction_id,
            state_file: args.state_file,
        }
    }
}

impl From<CliArgs> for AppConfig {
    fn from(args: CliArgs) -> Self {
        Self {
            api_key: args.api_key,
            faction_id: args.faction_id,
            state_path: args.state_file,
        }
    }
}

async fn run_bot(discord_token: Option<String>, check_interval: Option<u64>) -> Result<()> {
    #[allow(unused_imports)]
    use discord::{format_hits_message, Bot, Scheduler, Storage};
    use serenity::all::GatewayIntents;
    use serenity::client::ClientBuilder;
    use std::env;
    use tracing_subscriber::prelude::*;

    let token = discord_token
        .or_else(|| env::var("DISCORD_BOT_TOKEN").ok())
        .ok_or_else(|| anyhow!("Discord bot token not provided. Set DISCORD_BOT_TOKEN environment variable or use --discord-token"))?;

    let default_api_key = env::var("TORN_API_KEY")
        .ok()
        .filter(|k| !k.is_empty())
        .ok_or_else(|| {
            anyhow!("Torn API key not provided. Set TORN_API_KEY environment variable.")
        })?;

    let storage_path = dirs::config_dir()
        .map(|p| p.join("faction-hits").join("discord-storage.json"))
        .unwrap_or_else(|| PathBuf::from("discord-storage.json"));

    let storage = Storage::load(&storage_path).unwrap_or_default();

    let bot = Bot::new(default_api_key)
        .with_storage(storage)
        .with_check_interval(check_interval.unwrap_or(300));

    let storage = bot.storage.clone();
    let default_key = bot.default_api_key.clone();
    let interval = bot.check_interval_secs;

    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES;

    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init();

    let mut client = ClientBuilder::new(&token, intents)
        .event_handler(BotEventHandler)
        .await?;

    tracing::info!("Starting bot...");

    let storage_clone = storage.clone();
    tokio::spawn(async move {
        let scheduler = Scheduler::new(storage_clone, default_key, interval);
        scheduler
            .run(|_guild_id, _channel_id, _faction_id, _api_key| async {
                // TODO: Send message to Discord channel
            })
            .await;
    });

    client.start().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_args_from_clap_args() {
        let clap_args = ClapArgs {
            faction_id: Some(12345),
            state_file: Some(PathBuf::from("/tmp/state.json")),
        };
        let args: CliArgs = clap_args.into();
        assert_eq!(args.faction_id, Some(12345));
        assert_eq!(args.state_file, Some(PathBuf::from("/tmp/state.json")));
        assert_eq!(args.api_key, None);
    }

    #[test]
    fn test_cli_args_to_app_config() {
        let args = CliArgs {
            api_key: Some("test-key".to_string()),
            faction_id: Some(12345),
            state_file: Some(PathBuf::from("/tmp/state.json")),
        };
        let config: AppConfig = args.into();
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.faction_id, Some(12345));
        assert_eq!(config.state_path, Some(PathBuf::from("/tmp/state.json")));
    }
}
