use async_trait::async_trait;
use serenity::builder::CreateMessage;
use serenity::model::id::ChannelId;
use serenity::prelude::*;

use super::ScheduledTask;

/// リマインダータスク（例：1時間ごとにランダムなTipsを送信）
pub struct TipsReminderTask {
    channel_id: ChannelId,
    tips: Vec<String>,
}

impl TipsReminderTask {
    pub fn new(channel_id: u64) -> Self {
        Self {
            channel_id: ChannelId::new(channel_id),
            tips: vec![
                "💡 Tip: `/help` コマンドで利用可能なコマンドを確認できます！".to_string(),
                "💡 Tip: 問題が発生した場合は管理者にお知らせください。".to_string(),
                "💡 Tip: 定期的にチャンネルルールを確認しましょう！".to_string(),
                "💡 Tip: ボットのコマンドは随時更新されています。".to_string(),
            ],
        }
    }
}

#[async_trait]
impl ScheduledTask for TipsReminderTask {
    fn name(&self) -> &str {
        "TipsReminderTask"
    }

    fn interval_secs(&self) -> u64 {
        3600 // 1時間
    }

    async fn execute(&self, ctx: &Context) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use rand::seq::SliceRandom;
        
        // thread_rngを使用する前にtipを選択
        let tip = {
            let mut rng = rand::thread_rng();
            self.tips.choose(&mut rng).cloned()
        };
        
        if let Some(tip) = tip {
            let builder = CreateMessage::new().content(&tip);
            self.channel_id.send_message(&ctx.http, builder).await?;
            
            println!("[{}] Tip sent: {}", self.name(), tip);
        }
        
        Ok(())
    }
}