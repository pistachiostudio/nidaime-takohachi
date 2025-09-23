use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::*;

pub async fn run(
    ctx: &Context,
    interaction: &CommandInteraction,
) -> Result<(), Box<dyn std::error::Error>> {
    let ticker = interaction
        .data
        .options
        .iter()
        .find(|opt| opt.name == "ticker")
        .and_then(|opt| opt.value.as_str())
        .unwrap_or("^N225");

    let response_content = match crate::utils::get_stock_price(ticker).await {
        Ok((ratio_str, price)) => {
            let ticker_name = match ticker {
                "^N225" => "日経平均株価",
                "^DJI" => "ダウ平均株価",
                "^GSPC" => "S&P 500",
                "^IXIC" => "NASDAQ総合指数",
                _ => ticker,
            };
            format!(
                "📈 **デバッグ: 株価情報取得テスト**\n\n**{}**: ${} {}",
                ticker_name, price, ratio_str
            )
        }
        Err(e) => format!("❌ 株価情報の取得に失敗しました: {}", e),
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
    CreateCommand::new("debug_stock")
        .description("株価情報取得機能のテスト（デバッグ用）")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "ticker",
                "ティッカーシンボル (例: ^N225=日経平均, ^DJI=ダウ平均)",
            )
            .required(false),
        )
}
