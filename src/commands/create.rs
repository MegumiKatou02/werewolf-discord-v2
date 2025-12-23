use crate::bot::BotData;
use crate::commands::{CommandFuture, SlashCommand};
use crate::game::room::spawn_room;
use crate::game::RoomEvent;
use serenity::all::*;
use std::sync::Arc;

pub struct CreateCommand;

impl SlashCommand for CreateCommand {
    fn name(&self) -> &'static str {
        "masoi-create"
    }

    fn run(&self, ctx: Context, cmd: CommandInteraction, data: Arc<BotData>) -> CommandFuture {
        Box::pin(async move {
            let guild_id = match cmd.guild_id {
                Some(id) => id,
                None => {
                    cmd.create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("❌ Lệnh này chỉ sử dụng được trong server.")
                                .ephemeral(true),
                        ),
                    )
                    .await?;
                    return Ok(());
                }
            };

            let mut registry = data.room_registry.write().await;

            if let Some(existing_handle) = registry.get(&guild_id) {
                if !existing_handle.sender.is_closed() {
                    cmd.create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("❌ Đã có một phòng chơi đang hoạt động trong server này!")
                                .ephemeral(true),
                        ),
                    )
                    .await?;
                    return Ok(());
                } else {
                }
            }

            let host_id = cmd.user.id;
            let channel_id = cmd.channel_id;

            let new_handle = spawn_room(
                guild_id,
                host_id,
                channel_id,
                &ctx.clone(),
                data.roles_json.clone(),
            );

            let (tx, rx) = tokio::sync::oneshot::channel();

            let avatar_url = cmd.user.face();

            let _ = new_handle.sender.send(RoomEvent::JoinRequest {
                user_id: host_id,
                name: cmd.user.name.clone(),
                avatar_url,
                channel_id: channel_id,
                reply: tx,
            });

            let _ = rx.await;

            registry.insert(guild_id, new_handle);

            let mut player_registry = data.player_registry.write().await;
            player_registry.insert(host_id, guild_id);
            drop(player_registry);

            let embed = CreateEmbed::new()
                .color(0x3498db)
                .title("🎮 PHÒNG CHƠI MA SÓI MỚI")
                .description("```🔌 Phòng đã được tạo thành công!```")
                .field("👑 Chủ Phòng", format!("<@{}>", host_id), true)
                .field("👥 Số Người Chơi", "1/36", true)
                .field("⌛ Trạng Thái", "Đang chờ", true)
                .footer(CreateEmbedFooter::new(
                    "💡 Sử dụng /masoi-join để tham gia phòng",
                ))
                .timestamp(Timestamp::now());

            cmd.create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().add_embed(embed),
                ),
            )
            .await?;

            Ok(())
        })
    }
}
