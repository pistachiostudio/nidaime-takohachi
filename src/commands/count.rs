use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    // Defer the response to have more time for processing
    interaction.defer(&ctx.http).await?;
    
    // Get the limit from command options, default to 1000
    let limit = interaction
        .data
        .options
        .iter()
        .find(|opt| opt.name == "limit")
        .and_then(|opt| opt.value.as_i64())
        .unwrap_or(1000) as usize;
    
    let channel_id = interaction.channel_id;
    let mut total_count = 0;
    let mut last_message_id: Option<MessageId> = None;
    
    // Discord API limits to 100 messages per request, so we need to paginate
    loop {
        let mut messages_request = GetMessages::new().limit(100);
        
        if let Some(last_id) = last_message_id {
            messages_request = messages_request.before(last_id);
        }
        
        let messages = channel_id.messages(&ctx.http, messages_request).await?;
        
        if messages.is_empty() {
            break;
        }
        
        total_count += messages.len();
        last_message_id = messages.last().map(|m| m.id);
        
        // If we got fewer than 100 messages, we've reached the end
        if messages.len() < 100 {
            break;
        }
        
        // Stop if we've reached the user-specified limit
        if total_count >= limit {
            break;
        }
    }
    
    let response_text = if total_count >= limit {
        format!("このチャンネルには{}件以上のメッセージがあります（取得上限: {}件）", limit, limit)
    } else {
        format!("このチャンネルのメッセージ数: **{}件**", total_count)
    };
    
    interaction
        .edit_response(&ctx.http, EditInteractionResponse::new().content(response_text))
        .await?;
    
    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("count")
        .description("チャンネル内のメッセージ数を取得します")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Integer,
                "limit",
                "取得するメッセージの最大数（デフォルト: 1000）"
            )
            .min_int_value(1)
            .max_int_value(10000)
            .required(false)
        )
}