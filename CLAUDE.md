# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Nidaime Takohachi (二代目タコ八) is a Discord bot written in Rust using the Serenity framework. It provides slash commands and scheduled tasks for Discord servers.

## Language Requirements

- All user-facing responses and messages should be in Japanese (日本語)
- Error messages shown to Discord users should be in Japanese
- Code comments and internal documentation can remain in English

## Development Commands

### Build
```bash
cargo build           # Debug build
cargo build --release # Release build for production
```

### Testing
```bash
cargo test --all      # Run all tests
cargo test <test_name> # Run specific test
```

### Linting and Formatting
```bash
cargo fmt --all -- --check  # Check formatting
cargo fmt --all             # Auto-format code
cargo clippy --all -- -D warnings  # Lint code with clippy
```

### Check Code
```bash
cargo check --all    # Quick compilation check without producing binaries
```

### Running Locally
```bash
# First, create config.json from config.example.json and add Discord token
cp config.example.json config.json
# Then run the bot
cargo run
```

## Architecture

### Core Structure
- **Entry Point**: `src/main.rs` - Initializes the bot, handles Discord events, and registers slash commands with the guild
- **Event Handler**: Implements Serenity's `EventHandler` trait to process Discord interactions and ready events
- **Configuration**: `src/config.rs` loads settings from `config.json` including Discord token, guild ID, and channel configurations

### Command System
All slash commands are in `src/commands/`:
- Commands are registered in the `ready` event handler in `main.rs`
- Each command module exports a `run` function that handles the interaction
- Command modules must be added to `src/commands/mod.rs` with `pub mod <module_name>;`
- Commands handle their own response creation using Discord's interaction API

### Scheduled Tasks
Located in `src/scheduled_tasks/`:
- **Task Manager**: `mod.rs` contains `ScheduledTaskManager` that spawns and manages background tasks
- **Daily Tasks**: `daily_morning_task.rs` - Sends scheduled messages to configured channels
- **Message Cleanup**: `delete_message.rs` - Handles delayed message deletion
- Tasks use `tokio::spawn` for async execution and `tokio::time::interval` for scheduling

### Utilities
`src/utils.rs` provides helper functions for:
- Date/time operations with Tokyo timezone
- Discord message formatting
- Random selection from lists

## Adding New Features

### New Slash Command
1. Create new file in `src/commands/` (e.g., `mycommand.rs`)
2. Add `pub mod mycommand;` to `src/commands/mod.rs`
3. Implement the command handler with signature: `pub async fn run(ctx: &Context, command: &CommandInteraction) -> Result<(), Box<dyn std::error::Error>>`
4. Add command case to the match statement in `src/main.rs::interaction_create()`
5. Register the command in `src/main.rs::ready()` using `CreateCommand::new()`

### New Scheduled Task
1. Create new file in `src/scheduled_tasks/`
2. Implement task with async function
3. Add task initialization to `ScheduledTaskManager::new()` in `src/scheduled_tasks/mod.rs`
4. Use `tokio::spawn` and `tokio::time::interval` for scheduling

## Configuration

The bot requires a `config.json` file with:
- `discord_token`: Bot authentication token
- `guild_id`: Discord server ID for command registration
- `morning_greeting_channel_id`: Channel for daily messages
- `admin_channel_id`: Channel for admin notifications
- Additional feature-specific settings

## Deployment

For production deployment, use systemd service:
- Build with `cargo build --release`
- Follow `SYSTEMD_SETUP.md` for service configuration
- Binary location: `/opt/takohachi/nidaime-takohachi`
- Config location: `/opt/takohachi/config.json`
- Service management: `sudo systemctl {start|stop|restart|status} takohachi`