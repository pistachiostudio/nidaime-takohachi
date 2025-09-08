use async_trait::async_trait;
use serenity::builder::CreateMessage;
use serenity::model::id::ChannelId;
use serenity::prelude::*;

use super::ScheduledTask;

/// ステータスレポートタスク（例：30分ごとにボットのステータスを報告）
pub struct StatusReportTask {
    channel_id: ChannelId,
}

impl StatusReportTask {
    pub fn new(channel_id: u64) -> Self {
        Self {
            channel_id: ChannelId::new(channel_id),
        }
    }
}

#[async_trait]
impl ScheduledTask for StatusReportTask {
    fn name(&self) -> &str {
        "StatusReportTask"
    }

    fn interval_secs(&self) -> u64 {
        1800 // 30分
    }

    async fn execute(&self, ctx: &Context) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let current_user = ctx.cache.current_user().clone();
        let guild_count = ctx.cache.guilds().len();
        
        let status_message = format!(
            "📊 **ステータスレポート**\n\
            ボット名: {}\n\
            サーバー数: {}\n\
            稼働状態: 正常",
            current_user.name,
            guild_count
        );
        
        let builder = CreateMessage::new().content(&status_message);
        self.channel_id.send_message(&ctx.http, builder).await?;
        
        println!("[{}] Status report sent", self.name());
        
        Ok(())
    }
}