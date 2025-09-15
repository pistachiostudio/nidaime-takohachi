pub mod daily_morning_task;
pub mod delete_message;
pub mod periodic_message;

use std::time::Duration;

use async_trait::async_trait;
use serenity::prelude::*;
use tokio::time::sleep;

use crate::config::ScheduledTasksConfig;

pub use daily_morning_task::DailyMorningTask;
pub use delete_message::DeleteMessageTask;
pub use periodic_message::PeriodicMessageTask;

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
            loop {
                // 次回実行までの秒数を取得
                let interval = task.interval_secs();

                println!("[{}] Next execution in {} seconds", task.name(), interval);

                // 指定された間隔で待機
                sleep(Duration::from_secs(interval)).await;

                // タスクを実行
                if let Err(e) = task.execute(&ctx_clone).await {
                    eprintln!("[{}] Error during execution: {:?}", task.name(), e);
                }

                // DailyMorningTaskのような時刻ベースのタスクは、
                // 実行後に次回実行時刻を再計算する必要がある
                // interval_secs()を毎回呼び出すことで対応
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

        println!(
            "Created {} scheduled tasks for channel {}",
            tasks.len(),
            channel_id
        );
    } else {
        println!("scheduled_channel_id not found in config. No scheduled tasks will be created.");
    }

    // 自動メッセージ削除タスクを追加
    if config.enable_delete_message_task && !config.delete_message_channels.is_empty() {
        tasks.push(Box::new(DeleteMessageTask::with_settings(
            config.delete_message_channels.clone(),
        )));
        println!(
            "DeleteMessageTask has been enabled with {} channels",
            config.delete_message_channels.len()
        );
    }

    // 日本時間AM 7:00の定期タスクを追加
    if let Some(morning_task_config) = &config.daily_morning_task {
        if morning_task_config.enabled {
            let mut task = DailyMorningTask::new(morning_task_config.channel_id);
            if let Some(api_key) = &morning_task_config.gemini_api_key {
                task = task.with_api_key(api_key.clone());
            }
            tasks.push(Box::new(task));
            println!(
                "DailyMorningTask has been enabled for channel {} at 7:00 AM JST",
                morning_task_config.channel_id
            );
        }
    }

    if !tasks.is_empty() {
        println!("Total {} scheduled tasks created", tasks.len());
    }

    tasks
}
