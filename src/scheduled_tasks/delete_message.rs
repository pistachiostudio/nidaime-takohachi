use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use serenity::builder::GetMessages;
use serenity::model::id::ChannelId;
use serenity::prelude::*;

use super::ScheduledTask;

/// 自動メッセージ削除タスク
pub struct DeleteMessageTask {
    channel_settings: HashMap<u64, u64>,
}

impl DeleteMessageTask {
    pub fn with_settings(channel_settings: HashMap<u64, u64>) -> Self {
        Self { channel_settings }
    }
}

#[async_trait]
impl ScheduledTask for DeleteMessageTask {
    fn name(&self) -> &str {
        "DeleteMessageTask"
    }

    fn interval_secs(&self) -> u64 {
        600 // 10分ごとに実行
    }

    async fn execute(&self, ctx: &Context) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("[{}] Message purge task is running.", self.name());

        // 現在のUNIX時間を取得
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        // 各チャンネルをループ処理
        for (&channel_id, &delete_after_secs) in &self.channel_settings {
            let channel = ChannelId::new(channel_id);

            // メッセージを取得して削除対象をカウント
            let mut pinned_count = 0;
            let mut messages_to_delete = Vec::new();
            let mut last_message_id = None;

            // ページネーションでメッセージを取得
            loop {
                let mut builder = GetMessages::new().limit(100);

                if let Some(last_id) = last_message_id {
                    builder = builder.before(last_id);
                }

                let messages = channel.messages(&ctx.http, builder).await?;

                if messages.is_empty() {
                    break;
                }

                for message in &messages {
                    // メッセージの作成時刻をUNIX時間に変換
                    let message_timestamp = message.timestamp.unix_timestamp() as u64;

                    // 古いメッセージかチェック（削除対象期間内）
                    // saturating_subを使用して、アンダーフローを防ぐ
                    if message_timestamp < now
                        && now.saturating_sub(message_timestamp) > delete_after_secs
                    {
                        if message.pinned {
                            pinned_count += 1;
                        } else {
                            messages_to_delete.push(message.id);
                        }
                    }
                    // 削除対象期間外（新しい）メッセージはスキップして継続
                }

                // 最後のメッセージIDを更新
                last_message_id = messages.last().map(|m| m.id);

                // 100件未満の場合は最後のページなので終了
                if messages.len() < 100 {
                    break;
                }
            }

            // ピン留めされていないメッセージを削除
            if !messages_to_delete.is_empty() {
                // 一度に削除できるのは100件まで
                for chunk in messages_to_delete.chunks(100) {
                    channel.delete_messages(&ctx.http, chunk).await?;
                }
            }

            let channel_name = channel
                .to_channel(&ctx.http)
                .await
                .ok()
                .and_then(|ch| ch.guild())
                .map(|ch| ch.name.clone())
                .unwrap_or_else(|| format!("channel_{}", channel_id));

            println!(
                "[{}] Purged {} messages in {} (pinned: {})",
                self.name(),
                messages_to_delete.len(),
                channel_name,
                pinned_count
            );
        }

        println!("[{}] Message purge task is finished.", self.name());
        Ok(())
    }
}
