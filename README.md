# Faction Hits CLI

A CLI tool to track non-anonymous hits on faction members in Torn City.

## Features

- Fetches faction attacks from the Torn API
- Filters out anonymous (stealth) hits
- Stores last check timestamp in a JSON state file
- Only reports new hits since last run

## Installation

```bash
cargo build --release
```

The binary will be at `target/release/faction-hits`.

## Configuration

The API key can be provided in three ways (in order of priority):

1. `--api-key` command line argument
2. `TORN_API_KEY` or `TORN_KEY` environment variable
3. `.env` file in the current directory

### State File

By default, the state file is stored at:
- Linux/macOS: `~/.config/faction-hits/state.json`
- Windows: `%APPDATA%\faction-hits\state.json`

Use `--state-file` to specify a custom location.

## Usage

```bash
# With API key from environment
faction-hits

# With API key from command line
faction-hits --api-key YOUR_API_KEY

# With specific faction ID
faction-hits --api-key YOUR_API_KEY --faction-id 12345

# With custom state file
faction-hits --api-key YOUR_API_KEY --state-file /path/to/state.json
```

## API Key Requirements

The Torn API key needs the following access:
- `faction` -> `attacks` or `attacksfull`

This requires at least Limited Access level.

## Output Example

```
Fetching faction attacks since timestamp 1234567890...
Found 15 total attacks

=== 3 New Non-Anonymous Hits ===

1. Player1 (111) attacked Target1 (222)
   Result: Lost | Respect: 1.50 | Time: 2024-01-15 14:30:00

2. Player2 (333) attacked Target2 (444)
   Result: Hospitalized | Respect: 2.25 | Time: 2024-01-15 14:35:00

State updated. Last check timestamp: 1234567900
```

## Development

```bash
# Run tests
cargo test

# Format code
cargo fmt

# Run clippy
cargo clippy
```
