use async_trait::async_trait;
use chrono::{Datelike, Local, NaiveTime, Timelike};
use chrono_tz::Asia::Tokyo;
use chrono_tz::Tz;
use serenity::builder::CreateMessage;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use std::time::Duration;

use super::ScheduledTask;

/// 日本時間の毎日AM 7:00に実行されるタスク
pub struct DailyMorningTask {
    channel_id: ChannelId,
    message: String,
    hour: u32,
    minute: u32,
}

impl DailyMorningTask {
    pub fn new(channel_id: u64, message: String) -> Self {
        Self {
            channel_id: ChannelId::new(channel_id),
            message,
            hour: 7,
            minute: 0,
        }
    }

    pub fn with_time(channel_id: u64, message: String, hour: u32, minute: u32) -> Self {
        Self {
            channel_id: ChannelId::new(channel_id),
            message,
            hour,
            minute,
        }
    }

    /// 次回実行時刻（日本時間）までの秒数を計算
    fn seconds_until_next_execution(&self) -> u64 {
        let jst: Tz = Tokyo;
        let now_jst = chrono::Utc::now().with_timezone(&jst);
        
        let target_time = NaiveTime::from_hms_opt(self.hour, self.minute, 0)
            .expect("Invalid time specified");
        
        let today_target = now_jst
            .date_naive()
            .and_time(target_time)
            .and_local_timezone(jst)
            .unwrap();
        
        let next_execution = if now_jst >= today_target {
            // 今日の実行時刻を過ぎている場合は明日の同時刻
            today_target + chrono::Duration::days(1)
        } else {
            // まだ今日の実行時刻になっていない
            today_target
        };
        
        let duration = next_execution.signed_duration_since(now_jst);
        duration.num_seconds() as u64
    }
}

#[async_trait]
impl ScheduledTask for DailyMorningTask {
    fn name(&self) -> &str {
        "DailyMorningTask"
    }

    fn interval_secs(&self) -> u64 {
        // 次回実行までの秒数を返す
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
        
        let builder = CreateMessage::new().content(&self.message);
        self.channel_id.send_message(&ctx.http, builder).await?;
        
        println!(
            "[{}] Message sent successfully to channel {}",
            self.name(),
            self.channel_id
        );
        
        Ok(())
    }
}