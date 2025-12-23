use serenity::all::{CreateAttachment, CreateEmbed};
use std::path::Path;

pub struct WerewolfEmbed {
    pub embed: CreateEmbed,
    pub attachment: CreateAttachment,
}

pub async fn create_werewolf_embed(
    file_name: &str,
    title: &str,
    description: &str,
) -> serenity::Result<WerewolfEmbed> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(file_name);

    let attachment = CreateAttachment::path(&path).await?;

    let embed = CreateEmbed::new()
        .title(title)
        .description(description)
        .color(0x00ae86)
        .image(format!("attachment://{}", file_name));

    Ok(WerewolfEmbed { embed, attachment })
}
