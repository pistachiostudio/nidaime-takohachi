mod commands;
mod config;
mod scheduled_tasks;

use config::Config;

use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::Interaction;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            // println!("Received command interaction: {command:#?}");

            let content = match command.data.name.as_str() {
                "ping" => Some(commands::ping::run(&command.data.options())),
                "modal" => {
                    commands::modal::run(&ctx, &command).await.unwrap();
                    None
                }
                "mt" => {
                    commands::marimo::run(&ctx, &command).await.unwrap();
                    None
                }
                "count" => {
                    commands::count::run(&ctx, &command).await.unwrap();
                    None
                }
                "debug_weather" => {
                    commands::debug_weather::run(&ctx, &command).await.unwrap();
                    None
                }
                "debug_stock" => {
                    commands::debug_stock::run(&ctx, &command).await.unwrap();
                    None
                }
                "dic" => {
                    commands::dic::run(&ctx, &command).await.unwrap();
                    None
                }
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // 設定ファイルを読み込む
        let config = Config::load().expect("Failed to load config.json");
        let guild_id = GuildId::new(config.guild_id);

        // Guild コマンドをセットする
        let mut command_list = vec![
            commands::ping::register(),
            commands::modal::register(),
            commands::count::register(),
            commands::marimo::register(),
        ];

        // dic コマンドを条件付きで追加
        if config.dic.is_some() {
            command_list.push(commands::dic::register());
        }

        // デバッグコマンドを条件付きで追加
        if config.debug_slash_commands {
            command_list.push(commands::debug_weather::register());
            command_list.push(commands::debug_stock::register());
        }

        let commands = guild_id.set_commands(&ctx.http, command_list).await;

        println!("I now have the following guild slash commands: {commands:#?}");

        // Global コマンドを作成する場合は以下のコメントアウトを外して Command をインポートする
        // let guild_command =
        //     Command::create_global_command(&ctx.http, commands::wonderful_command::register())
        //         .await;

        // println!("I created the following global slash command: {guild_command:#?}");

        // スケジュールタスクを開始
        let tasks = scheduled_tasks::create_scheduled_tasks(&config.scheduled_tasks);
        scheduled_tasks::start_scheduled_tasks(ctx.clone(), tasks).await;
        println!("Scheduled tasks have been started.");
    }
}

#[tokio::main]
async fn main() {
    // 設定ファイルを読み込む
    let config = Config::load().expect(
        "Failed to load config.json. Please create config.json based on config.example.json",
    );
    let token = config.discord_token.clone();

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
