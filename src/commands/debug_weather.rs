use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::*;

pub async fn run(
    ctx: &Context,
    interaction: &CommandInteraction,
) -> Result<(), Box<dyn std::error::Error>> {
    let citycode = interaction
        .data
        .options
        .iter()
        .find(|opt| opt.name == "citycode")
        .and_then(|opt| opt.value.as_str())
        .unwrap_or("130010");

    let response_content = match crate::utils::get_weather(citycode).await {
        Ok(weather_info) => format!("🌤️ **デバッグ: 天気情報取得テスト**\n\n{}", weather_info),
        Err(e) => format!("❌ 天気情報の取得に失敗しました: {}", e),
    };

    interaction
        .create_response(
            &ctx.http,
            serenity::builder::CreateInteractionResponse::Message(
                serenity::builder::CreateInteractionResponseMessage::new()
                    .content(response_content),
            ),
        )
        .await?;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("debug_weather")
        .description("天気情報取得機能のテスト（デバッグ用）")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "citycode",
                "都市コード (例: 130010=東京, 060010=山形)",
            )
            .required(false),
        )
}
