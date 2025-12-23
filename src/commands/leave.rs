use crate::bot::BotData;
use crate::commands::{CommandFuture, SlashCommand};
use crate::game::{LeaveResult, RoomEvent};
use serenity::all::*;
use std::sync::Arc;
use tokio::sync::oneshot;

pub struct LeaveCommand;

impl SlashCommand for LeaveCommand {
    fn name(&self) -> &'static str {
        "masoi-leave"
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
                                .content("Lệnh này chỉ dùng trong Server.")
                                .ephemeral(true),
                        ),
                    )
                    .await?;
                    return Ok(());
                }
            };

            let room_handle = {
                let registry = data.room_registry.read().await;
                match registry.get(&guild_id) {
                    Some(handle) => Some(handle.clone()),
                    None => None,
                }
            };

            let room_handle = match room_handle {
                Some(handle) => handle,
                None => {
                    cmd.create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("❌ Không tìm thấy phòng ma sói nào.")
                                .ephemeral(true),
                        ),
                    )
                    .await?;
                    return Ok(());
                }
            };

            let (tx, rx) = oneshot::channel();

            if room_handle
                .sender
                .send(RoomEvent::LeaveRequest {
                    user_id: cmd.user.id,
                    reply: tx,
                })
                .is_err()
            {
                cmd.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("❌ Phòng chơi đã đóng cửa.")
                            .ephemeral(true),
                    ),
                )
                .await?;
                data.room_registry.write().await.remove(&guild_id);
                return Ok(());
            }

            match rx.await {
                Ok(result) => match result {
                    LeaveResult::Success(count) => {
                        cmd.create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content(format!(
                                        "✅ Bạn đã rời khỏi phòng. Còn lại {} người.",
                                        count
                                    ))
                                    .ephemeral(false),
                            ),
                        )
                        .await?;
                    }
                    LeaveResult::NotJoined => {
                        cmd.create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("⚠️ Bạn chưa tham gia phòng chơi này.")
                                    .ephemeral(true),
                            ),
                        )
                        .await?;
                    }
                    LeaveResult::GameStarted => {
                        cmd.create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("⚠️ Trò chơi đã bắt đầu, không thể rời.")
                                    .ephemeral(true),
                            ),
                        )
                        .await?;
                    }
                    LeaveResult::RoomEmpty => {
                        cmd.create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("✅ Bạn đã rời phòng. Phòng trống nên đã bị hủy.")
                                    .ephemeral(false),
                            ),
                        )
                        .await?;

                        let _ = cmd
                            .channel_id
                            .say(
                                &ctx.http,
                                "🗑️ Không còn ai trong phòng nên phòng Ma Sói đã bị hủy.",
                            )
                            .await;

                        let mut registry = data.room_registry.write().await;
                        registry.remove(&guild_id);
                        tracing::info!("Room {} removed because it is empty", guild_id);
                    }
                },
                Err(_) => {
                    cmd.create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("❌ Lỗi xử lý.")
                                .ephemeral(true),
                        ),
                    )
                    .await?;
                }
            }

            Ok(())
        })
    }
}
