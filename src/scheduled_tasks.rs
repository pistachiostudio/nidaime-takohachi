use std::env;
use std::time::Duration;

use serenity::builder::CreateMessage;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use tokio::time::sleep;

pub async fn start_scheduled_tasks(ctx: Context) {
    tokio::spawn(async move {
        let channel_id = match env::var("SCHEDULED_CHANNEL_ID") {
            Ok(id) => match id.parse::<u64>() {
                Ok(parsed_id) => ChannelId::new(parsed_id),
                Err(e) => {
                    println!("Error parsing SCHEDULED_CHANNEL_ID: {}", e);
                    return;
                }
            },
            Err(_) => {
                println!(
                    "SCHEDULED_CHANNEL_ID not found in environment variables. Scheduled tasks will not run."
                );
                return;
            }
        };

        loop {
            // interval ではなく sleep を使うことで一定間隔での実行ができる (なんで?)
            sleep(Duration::from_secs(600)).await; // 10分待機

            let message = "⏰ 定期メッセージ: 10分が経過しました！";

            let builder = CreateMessage::new().content(message);

            if let Err(why) = channel_id.send_message(&ctx.http, builder).await {
                println!("Error sending scheduled message: {:?}", why);
            } else {
                println!(
                    "Scheduled message sent successfully to channel {}",
                    channel_id
                );
            }
        }
    });
}
