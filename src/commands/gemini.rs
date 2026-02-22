use reqwest::Client;
use serde::Deserialize;
use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::config::Config;

const DEFAULT_CHARACTER: &str =
    "あなたはチャットコミュニティのみんなに愛されるBotです。みんなからくるいろんな質問にバッチリ答えてね。";
const GEMINI_MODEL: &str = "gemini-2.5-pro";

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Content,
}

#[derive(Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Deserialize)]
struct Part {
    text: String,
}

pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    interaction.defer(&ctx.http).await?;

    let key = interaction
        .data
        .options
        .iter()
        .find(|opt| opt.name == "key")
        .and_then(|opt| opt.value.as_str())
        .unwrap_or("")
        .to_string();

    let character = interaction
        .data
        .options
        .iter()
        .find(|opt| opt.name == "character")
        .and_then(|opt| opt.value.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| DEFAULT_CHARACTER.to_string());

    // Load config in a block so the non-Send error is dropped before any await
    let api_key = {
        match Config::load() {
            Ok(c) => c.gemini.map(|g| g.api_key),
            Err(e) => {
                println!("Failed to load config: {}", e);
                None
            }
        }
    };

    let api_key = match api_key {
        Some(k) => k,
        None => {
            interaction
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content(":warning: Gemini API キーが設定されていません。"),
                )
                .await?;
            return Ok(());
        }
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        GEMINI_MODEL, api_key
    );

    let payload = serde_json::json!({
        "contents": [
            {
                "parts": [
                    {"text": character},
                    {"text": key}
                ]
            }
        ]
    });

    let client = Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await;

    let answer = match response {
        Ok(res) => match res.json::<GeminiResponse>().await {
            Ok(gemini_res) => {
                match gemini_res
                    .candidates
                    .into_iter()
                    .next()
                    .and_then(|c| c.content.parts.into_iter().next())
                    .map(|p| p.text)
                {
                    Some(text) => text,
                    None => {
                        interaction
                            .edit_response(
                                &ctx.http,
                                EditInteractionResponse::new().content(
                                    ":warning: Gemini APIからの応答を解析できませんでした。",
                                ),
                            )
                            .await?;
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                println!("Failed to parse Gemini response: {}", e);
                interaction
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new()
                            .content(":warning: Gemini APIのレスポンスの解析に失敗しました。"),
                    )
                    .await?;
                return Ok(());
            }
        },
        Err(e) => {
            println!("Gemini API request failed: {}", e);
            interaction
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content(":warning: Gemini APIへのリクエストに失敗しました。"),
                )
                .await?;
            return Ok(());
        }
    };

    let character_display = if character == DEFAULT_CHARACTER {
        "Default".to_string()
    } else {
        character
    };

    let embed = CreateEmbed::new()
        .title(format!("Q. {}", key))
        .description(answer)
        .colour(Colour::DARK_GREEN)
        .footer(CreateEmbedFooter::new(format!(
            " Model: {}\n🪀 キャラ設定: {}",
            GEMINI_MODEL, character_display
        )));

    interaction
        .edit_response(&ctx.http, EditInteractionResponse::new().embed(embed))
        .await?;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("gemini")
        .description("Geminiに質問をしましょう！")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "key", "質問内容").required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "character",
                "Geminiに性格やキャラを与えることができます。必ず「あなたは～です。」と書いてください。",
            )
            .required(false),
        )
}
