use async_trait::async_trait;
use chrono::{Datelike, NaiveTime, Timelike};
use chrono_tz::Asia::Tokyo;
use chrono_tz::Tz;
use rand::seq::SliceRandom;
use serenity::builder::{CreateEmbed, CreateMessage};
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

use super::ScheduledTask;
use nidaime_takohachi::utils;

pub struct DailyMorningTask {
    channel_id: ChannelId,
    hour: u32,
    minute: u32,
    gemini_api_key: Option<String>,
}

impl DailyMorningTask {
    pub fn new(channel_id: u64) -> Self {
        Self {
            channel_id: ChannelId::new(channel_id),
            hour: 7,
            minute: 0,
            gemini_api_key: None,
        }
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.gemini_api_key = Some(api_key);
        self
    }

    fn seconds_until_next_execution(&self) -> u64 {
        let jst: Tz = Tokyo;
        let now_jst = chrono::Utc::now().with_timezone(&jst);

        let target_time =
            NaiveTime::from_hms_opt(self.hour, self.minute, 0).expect("Invalid time specified");

        let today_target = now_jst
            .date_naive()
            .and_time(target_time)
            .and_local_timezone(jst)
            .unwrap();

        let next_execution = if now_jst >= today_target {
            today_target + chrono::Duration::days(1)
        } else {
            today_target
        };

        let duration = next_execution.signed_duration_since(now_jst);
        duration.num_seconds() as u64
    }

    async fn build_morning_message(&self) -> String {
        let jst: Tz = Tokyo;
        let now_jst = chrono::Utc::now().with_timezone(&jst);
        let month = now_jst.month();
        let day = now_jst.day();

        let what_today = utils::get_what_today(month, day).await;

        let tokyo_weather = match utils::get_weather("130010").await {
            Ok(weather_info) => weather_info,
            Err(e) => format!("東京の天気情報を取得できませんでした: {}", e),
        };
        let yamagata_weather = match utils::get_weather("060010").await {
            Ok(weather_info) => weather_info,
            Err(e) => format!("山形の天気情報を取得できませんでした: {}", e),
        };

        let trivia = if let Some(api_key) = &self.gemini_api_key {
            match utils::get_trivia(api_key).await {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[{}] Gemini API error: {}", self.name(), e);
                    "今日の雑学: 知識は力なり！".to_string()
                }
            }
        } else {
            "今日の雑学: APIキーが設定されていません".to_string()
        };

        let market_data = vec![
            ("USDJPY=X", "USD/JPY", "円", "💰"),
            ("^N225", "日経225", "円", "🇯🇵"),
            ("^GSPC", "S&P500", "pt", "🇺🇸"),
            ("^IXIC", "NASDAQ", "pt", "🇺🇸"),
            ("3399.T", "丸千代山岡家", "円", "🍜"),
            ("9023.T", "東京地下鉄", "円", "🚇"),
        ];

        let mut market_lines = Vec::new();
        for (ticker, name, unit, icon) in market_data {
            match utils::get_stock_price(ticker).await {
                Ok((ratio, price)) => {
                    market_lines.push(format!(
                        "- {} **{}:** {}{} {}",
                        icon, name, price, unit, ratio
                    ));
                }
                Err(_) => {
                    market_lines.push(format!("- {} **{}:** データ取得失敗", icon, name));
                }
            }
            sleep(Duration::from_millis(500)).await;
        }

        let market_text = if market_lines.is_empty() {
            "市場データを取得できませんでした".to_string()
        } else {
            format!("{}\n※()内は前日比。", market_lines.join("\n"))
        };

        format!(
            "### 💡 今日はなんの日？\n{}\n\n### 📚 今日の雑学\n{}\n(Powered by [Gemini](https://ai.google.dev/gemini-api/docs/models))\n\n### 💹 相場\n{}\n\n### ⛅ 今日の天気\n{}\n{}",
            what_today, trivia, market_text, tokyo_weather, yamagata_weather
        )
    }
}

#[async_trait]
impl ScheduledTask for DailyMorningTask {
    fn name(&self) -> &str {
        "DailyMorningTask"
    }

    fn interval_secs(&self) -> u64 {
        self.seconds_until_next_execution()
    }

    async fn execute(&self, ctx: &Context) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let jst: Tz = Tokyo;
        let now_jst = chrono::Utc::now().with_timezone(&jst);

        println!(
            "[{}] Executing at Japan time: {}/{} {:02}:{:02}",
            self.name(),
            now_jst.month(),
            now_jst.day(),
            now_jst.hour(),
            now_jst.minute()
        );

        let greetings = ["おざし。", "おざす。", "お。", "おはようございます。"];
        let greeting = greetings
            .choose(&mut rand::thread_rng())
            .unwrap_or(&"おはようございます。");
        let title = format!(
            "{}{}月{}日 朝の{}時です。",
            greeting,
            now_jst.month(),
            now_jst.day(),
            self.hour
        );

        let description = self.build_morning_message().await;

        let embed = CreateEmbed::new()
            .title(title)
            .description(description)
            .color(0x00ff00);

        let builder = CreateMessage::new().embed(embed);
        self.channel_id.send_message(&ctx.http, builder).await?;

        println!(
            "[{}] Message sent successfully to channel {}",
            self.name(),
            self.channel_id
        );

        Ok(())
    }
}
