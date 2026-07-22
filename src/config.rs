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
    /// 設定を読み込む
    ///
    /// 環境変数 `CONFIG_JSON` が設定されていればそちらを優先し、
    /// なければ `config.json` ファイルから読み込む（ローカル開発用）。
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        if let Ok(json) = std::env::var("CONFIG_JSON") {
            let config: Config = serde_json::from_str(&json)?;
            config.write_service_account_key_if_present()?;
            return Ok(config);
        }
        Self::load_from_path("config.json")
    }

    /// `GOOGLE_SERVICE_ACCOUNT_KEY_JSON` 環境変数が設定されていれば、
    /// `dic.service_account_key_path` が指すパスにその内容を書き出す。
    /// Railway のようなコンテナ環境では鍵ファイルを直接配置できないための対応。
    fn write_service_account_key_if_present(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (Some(dic), Ok(key_json)) = (
            &self.dic,
            std::env::var("GOOGLE_SERVICE_ACCOUNT_KEY_JSON"),
        ) else {
            return Ok(());
        };
        if let Some(parent) = Path::new(&dic.service_account_key_path).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&dic.service_account_key_path, key_json)?;
        Ok(())
    }

    /// 指定されたパスから設定ファイルを読み込む
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&contents)?;
        Ok(config)
    }
}
