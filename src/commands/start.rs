use crate::bot::BotData;
use crate::commands::{CommandFuture, SlashCommand};
use serenity::all::*;
use std::sync::Arc;

pub struct StartCommand;

impl SlashCommand for StartCommand {
    fn name(&self) -> &'static str {
        "masoi-start"
    }

    fn run(&self, ctx: Context, cmd: CommandInteraction, data: Arc<BotData>) -> CommandFuture {
        Box::pin(async move {
            let guild_id = match cmd.guild_id {
                Some(id) => id,
                None => return Ok(()),
            };

            let has_room = data.room_registry.read().await.contains_key(&guild_id);
            if !has_room {
                cmd.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("Chưa có phòng chơi.")
                            .ephemeral(true),
                    ),
                )
                .await?;
                return Ok(());
            }

            let row = CreateActionRow::Buttons(vec![
                CreateButton::new("start_default")
                    .label("Dùng vai trò mặc định")
                    .style(ButtonStyle::Primary),
                CreateButton::new("start_custom_json")
                    .label("Tuỳ chỉnh (JSON)")
                    .style(ButtonStyle::Secondary),
                CreateButton::new("start_custom_name")
                    .label("Tuỳ chỉnh (Tên)")
                    .style(ButtonStyle::Secondary),
            ]);

            cmd.create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("🎮 Chọn cách phân vai trò:")
                        .components(vec![row])
                        .ephemeral(true),
                ),
            )
            .await?;

            Ok(())
        })
    }
}
