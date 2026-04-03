# PR Review Plan - [#6] feat: add Discord bot for automated faction hits posting

## Issues to Address

- [ ] **Copilot: CLI argument parsing broken** → [Details](./.opencode/plan-details/pr-review-001.md)
  - Args::parse() only knows about optional `bot` subcommand; missing CLI flags (`--faction-id`, `--state-file`, `--api-key`)
  - Running CLI with flags fails during initial parse with "unknown argument" error

- [ ] **Copilot: CLI missing --api-key argument** → [Details](./.opencode/plan-details/pr-review-002.md)
  - CLI no longer accepts `--api-key` argument (and the env-backed clap arg)
  - If passing key via CLI is still supported, reintroduce `api_key` option in parsed CLI args

- [ ] **Copilot: Bot functionality incomplete** → [Details](./.opencode/plan-details/pr-review-003.md)
  - Bot loop never posts hits to Discord (`// TODO: Send message to Discord channel`)
  - No updated `last_check_timestamp`/guild config persisted back to `discord-storage.json`
  - Restarts will likely re-post same hits

- [ ] **Copilot: Scheduler design issue** → [Details](./.opencode/plan-details/pr-review-004.md)
  - `Scheduler::check_all_guilds` computes `new_hits` but callback signature only receives IDs and API key
  - Impossible for caller to format/post actual hit details without re-fetching
  - Consider passing `&[FactionAttack]`/`Vec<FactionAttack>` or preformatted message to callback

- [ ] **Copilot: Message formatting bug** → [Details](./.opencode/plan-details/pr-review-005.md)
  - `format_hits_message` always renders header as "... Hits ..."
  - Unit test expects singular form ("1 New Non-Anonymous Hit")
  - Tests will fail; adjust header to use correct singular/plural

- [ ] **Copilot: Storage error handling** → [Details](./.opencode/plan-details/pr-review-006.md)
  - `Storage::load` silently returns `Ok(default)` on JSON parse errors
  - Drops all saved guild configuration without surfacing problem
  - Should return parse error or at least log parse failure

- [ ] **Copilot: Security concern with API keys** → [Details](./.opencode/plan-details/pr-review-007.md)
  - `Storage::save` writes `discord-storage.json` with per-guild Torn API keys in plaintext
  - Consider restricting file permissions on save or avoiding on-disk storage of secrets

## Resolved/Nice to Have

- [ ] **Copilot: Library restructuring looks good** → [Details](./.opencode/plan-details/pr-review-008.md)
  - Introduces `src/lib.rs` library entrypoint and updates binary to consume `faction_hits::*` exports
  - Adds initial Discord bot modules (storage, scheduler, command parsing, bot config)
  - New `faction-hits bot` entrypoint works as expected