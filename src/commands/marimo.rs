use chrono::{Datelike, Timelike, Utc};
use chrono_tz::Tz;
use rand::seq::SliceRandom;
use serenity::all::Colour;
use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    let now_utc = Utc::now();

    let est: Tz = "America/New_York".parse().unwrap();
    let cet: Tz = "Europe/Paris".parse().unwrap();
    let jst: Tz = "Asia/Tokyo".parse().unwrap();

    let marimo_time = now_utc.with_timezone(&est);
    let sopot_time = now_utc.with_timezone(&cet);
    let japan_time = now_utc.with_timezone(&jst);

    let marimo_time_str = format!(
        "{}/{} {}:{:02}",
        marimo_time.month(),
        marimo_time.day(),
        marimo_time.hour(),
        marimo_time.minute()
    );
    let sopot_time_str = format!(
        "{}/{} {}:{:02}",
        sopot_time.month(),
        sopot_time.day(),
        sopot_time.hour(),
        sopot_time.minute()
    );
    let japan_time_str = format!(
        "{}/{} {}:{:02}",
        japan_time.month(),
        japan_time.day(),
        japan_time.hour(),
        japan_time.minute()
    );

    let slot_list = ["🍒", "🔔", "🍉", "🍇", "🍋", "🐈", "🐬", "🦕", "🐢", "🐕"];
    let slot_left;
    let slot_center;
    let slot_right;
    {
        let mut rng = rand::thread_rng();
        slot_left = slot_list.choose(&mut rng).unwrap();
        slot_center = slot_list.choose(&mut rng).unwrap();
        slot_right = slot_list.choose(&mut rng).unwrap();
    }

    let mut embeds = vec![];

    let embed1 = CreateEmbed::new()
        .color(Colour::DARK_GREEN)
        .footer(CreateEmbedFooter::new(format!(
            "mt slot: {}{}{}",
            slot_left, slot_center, slot_right
        )))
        .description(format!(
            "marimo time = **{}**\nSopot time = **{}**\n(In Japan = {})",
            marimo_time_str, sopot_time_str, japan_time_str
        ));

    embeds.push(embed1);

    if slot_left == slot_center && slot_center == slot_right {
        let embed2 = CreateEmbed::new()
            .color(Colour::DARK_GREEN)
            .description(format!(
                "🎉Congratulations!! {} hits the Jackpot!!🎉",
                interaction.user.mention()
            ));
        embeds.push(embed2);
    }

    let builder =
        CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embeds(embeds));

    interaction.create_response(&ctx.http, builder).await?;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("mt").description("まりもたいむ")
}
