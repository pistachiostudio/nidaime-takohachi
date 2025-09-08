use async_trait::async_trait;
use serenity::builder::CreateMessage;
use serenity::model::id::ChannelId;
use serenity::prelude::*;

use super::ScheduledTask;

/// 10分ごとにメッセージを送信するタスク
pub struct PeriodicMessageTask {
    channel_id: ChannelId,
    message: String,
    interval: u64,
}

impl PeriodicMessageTask {
    pub fn new(channel_id: u64, message: String, interval_secs: u64) -> Self {
        Self {
            channel_id: ChannelId::new(channel_id),
            message,
            interval: interval_secs,
        }
    }
}

#[async_trait]
impl ScheduledTask for PeriodicMessageTask {
    fn name(&self) -> &str {
        "PeriodicMessageTask"
    }

    fn interval_secs(&self) -> u64 {
        self.interval
    }

    async fn execute(&self, ctx: &Context) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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