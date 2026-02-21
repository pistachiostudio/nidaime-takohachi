use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::config::Config;

pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    interaction.defer(&ctx.http).await?;

    let keyword = interaction
        .data
        .options
        .iter()
        .find(|opt| opt.name == "keyword")
        .and_then(|opt| opt.value.as_str())
        .unwrap_or("");

    // Load config in a block so the non-Send error is dropped before any await
    let config = {
        match Config::load() {
            Ok(c) => Some(c),
            Err(e) => {
                println!("Failed to load config: {}", e);
                None
            }
        }
    };
    let Some(config) = config else {
        interaction
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new()
                    .content(":warning: データの取得中にエラーが発生しました。"),
            )
            .await?;
        return Ok(());
    };

    let dic_config = match &config.dic {
        Some(c) => c,
        None => {
            interaction
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content(":warning: dic コマンドが設定されていません。"),
                )
                .await?;
            return Ok(());
        }
    };

    let result = nidaime_takohachi::google_sheets::search_trigger(
        &dic_config.service_account_key_path,
        &dic_config.spreadsheet_id,
        keyword,
    )
    .await;

    match result {
        Ok(Some(entry)) => {
            if !entry.response.is_empty() {
                interaction
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new().content(&entry.response),
                    )
                    .await?;
            } else {
                let mut embed = CreateEmbed::new().colour(Colour::DARK_BLUE);

                if !entry.title.is_empty() {
                    embed = embed.title(&entry.title);
                }
                if !entry.description.is_empty() {
                    embed = embed.description(format!(
                        "{}\n\n[Check DB]({})",
                        entry.description, dic_config.db_spreadsheet_url
                    ));
                }
                if !entry.thumbnail_url.is_empty() {
                    embed = embed.thumbnail(&entry.thumbnail_url);
                }
                if !entry.image_url.is_empty() {
                    embed = embed.image(&entry.image_url);
                }
                embed = embed.footer(CreateEmbedFooter::new(format!("Keyword: {}", keyword)));

                interaction
                    .edit_response(&ctx.http, EditInteractionResponse::new().embed(embed))
                    .await?;
            }
        }
        Ok(None) => {
            interaction
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content(format!(":warning: 「{}」は登録されていません。", keyword)),
                )
                .await?;
        }
        Err(e) => {
            println!("Error searching trigger: {}", e);
            interaction
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content(":warning: データの取得中にエラーが発生しました。"),
                )
                .await?;
        }
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("dic")
        .description("Trigger Commands")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "keyword",
                "キーワードを入力してください。例) genkai, 徳井病, gomi など",
            )
            .required(true),
        )
}
