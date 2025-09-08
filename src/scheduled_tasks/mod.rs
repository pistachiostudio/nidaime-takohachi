pub mod delete_message;
pub mod periodic_message;
pub mod status_report;
pub mod tips_reminder;

use std::time::Duration;

use async_trait::async_trait;
use serenity::prelude::*;
use tokio::time::sleep;

use crate::config::ScheduledTasksConfig;

pub use delete_message::DeleteMessageTask;
pub use periodic_message::PeriodicMessageTask;
pub use status_report::StatusReportTask;
pub use tips_reminder::TipsReminderTask;

/// スケジュールタスクのトレイト
#[async_trait]
pub trait ScheduledTask: Send + Sync {
    /// タスクの名前を返す
    fn name(&self) -> &str;

    /// タスクの実行間隔を返す（秒単位）
    fn interval_secs(&self) -> u64;

    /// タスクの実行処理
    async fn execute(&self, ctx: &Context) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 複数のスケジュールタスクを並行して実行する
pub async fn start_scheduled_tasks(ctx: Context, tasks: Vec<Box<dyn ScheduledTask>>) {
    println!("Starting {} scheduled tasks...", tasks.len());
    
    for task in tasks.into_iter() {
        let ctx_clone = ctx.clone();
        
        tokio::spawn(async move {
            println!(
                "[{}] Starting with interval: {} seconds",
                task.name(),
                task.interval_secs()
            );
            
            loop {
                // 指定された間隔で待機
                sleep(Duration::from_secs(task.interval_secs())).await;
                
                // タスクを実行
                if let Err(e) = task.execute(&ctx_clone).await {
                    eprintln!(
                        "[{}] Error during execution: {:?}",
                        task.name(),
                        e
                    );
                }
            }
        });
    }
}

/// 設定に基づいてスケジュールタスクを作成する関数
pub fn create_scheduled_tasks(config: &ScheduledTasksConfig) -> Vec<Box<dyn ScheduledTask>> {
    let mut tasks: Vec<Box<dyn ScheduledTask>> = Vec::new();
    
    // チャンネルIDが設定されている場合
    if let Some(channel_id) = config.scheduled_channel_id {
        // 10分ごとの定期メッセージ
        tasks.push(Box::new(PeriodicMessageTask::new(
            channel_id,
            "⏰ 定期メッセージ: 10分が経過しました！".to_string(),
            600,
        )));
        
        // 30分ごとのステータスレポート
        tasks.push(Box::new(StatusReportTask::new(channel_id)));
        
        // 1時間ごとのTipsリマインダー
        tasks.push(Box::new(TipsReminderTask::new(channel_id)));
        
        println!("Created {} scheduled tasks for channel {}", tasks.len(), channel_id);
    } else {
        println!("scheduled_channel_id not found in config. No scheduled tasks will be created.");
    }
    
    // 自動メッセージ削除タスクを追加
    if config.enable_delete_message_task && !config.delete_message_channels.is_empty() {
        tasks.push(Box::new(DeleteMessageTask::with_settings(
            config.delete_message_channels.clone()
        )));
        println!("DeleteMessageTask has been enabled with {} channels", config.delete_message_channels.len());
    }
    
    if !tasks.is_empty() {
        println!("Total {} scheduled tasks created", tasks.len());
    }
    
    tasks
}