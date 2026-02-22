use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub discord_token: String,
    pub guild_id: u64,
    #[serde(default)]
    pub scheduled_tasks: ScheduledTasksConfig,
    #[serde(default)]
    pub debug_slash_commands: bool,
    pub dic: Option<DicConfig>,
    pub gemini: Option<GeminiConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScheduledTasksConfig {
    pub scheduled_channel_id: Option<u64>,
    #[serde(default)]
    pub enable_delete_message_task: bool,
    #[serde(default)]
    pub delete_message_channels: HashMap<u64, u64>,
    pub daily_morning_task: Option<DailyMorningTaskConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyMorningTaskConfig {
    pub enabled: bool,
    pub channel_id: u64,
    pub gemini_api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicConfig {
    pub spreadsheet_id: String,
    pub service_account_key_path: String,
    pub db_spreadsheet_url: String,
}

impl Config {
    /// 設定ファイルを読み込む
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_from_path("config.json")
    }

    /// 指定されたパスから設定ファイルを読み込む
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&contents)?;
        Ok(config)
    }
}
