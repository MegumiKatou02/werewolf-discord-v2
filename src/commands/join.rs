use crate::bot::BotData;
use crate::commands::{CommandFuture, SlashCommand};
use crate::game::{JoinResult, RoomEvent};
use serenity::all::*;
use std::sync::Arc;
use tokio::sync::oneshot;

pub struct JoinCommand;

impl SlashCommand for JoinCommand {
    fn name(&self) -> &'static str {
        "masoi-join"
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

            let registry = data.room_registry.read().await;
            let room_handle = match registry.get(&guild_id) {
                Some(handle) => handle.clone(),
                None => {
                    cmd.create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("❌ Không có trò chơi ma sói nào đang chờ trong server.")
                                .ephemeral(true),
                        ),
                    )
                    .await?;
                    return Ok(());
                }
            };

            drop(registry);

            let player_registry = data.player_registry.read().await;

            if let Some(playing_guild_id) = player_registry.get(&cmd.user.id) {
                let msg = format!(
                    "❌ Bạn đang tham gia một ván game ở Server khác (ID: {}) rồi!\nHãy hoàn thành hoặc rời ván đó trước.",
                    playing_guild_id
                );
                cmd.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(msg)
                            .ephemeral(true),
                    ),
                )
                .await?;
                return Ok(());
            }
            drop(player_registry);

            let avatar_url = cmd.user.face();

            let (tx, rx) = oneshot::channel();

            let send_result = room_handle.sender.send(RoomEvent::JoinRequest {
                user_id: cmd.user.id,
                name: cmd.user.name.clone(),
                avatar_url: avatar_url,
                channel_id: cmd.channel_id,
                reply: tx,
            });

            if send_result.is_err() {
                cmd.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("❌ Phòng chơi đã bị đóng đột ngột.")
                            .ephemeral(true),
                    ),
                )
                .await?;
                return Ok(());
            }

            match rx.await {
                Ok(result) => match result {
                    JoinResult::Success(count) => {
                        let mut player_registry = data.player_registry.write().await;
                        player_registry.insert(cmd.user.id, guild_id);
                        drop(player_registry);

                        cmd.create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content(format!(
                                        "✅ <@{}> đã tham gia phòng! Hiện có {} người",
                                        cmd.user.id, count
                                    ))
                                    .ephemeral(false),
                            ),
                        )
                        .await?;
                    }
                    JoinResult::RoomFull => {
                        cmd.create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("❌ Đã quá giới hạn số lượng người tham gia.")
                                    .ephemeral(true),
                            ),
                        )
                        .await?;
                    }
                    JoinResult::AlreadyJoined => {
                        cmd.create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("⚠️ Bạn đã tham gia trò chơi rồi.")
                                    .ephemeral(true),
                            ),
                        )
                        .await?;
                    }
                    JoinResult::GameStarted => {
                        cmd.create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("⚠️ Trò chơi đã bắt đầu, không thể tham gia.")
                                    .ephemeral(true),
                            ),
                        )
                        .await?;
                    }
                    JoinResult::WrongChannel(true_channel_id, host_id) => {
                        let msg = format!(
                            "⚠️ Trò chơi bắt đầu ở kênh <#{}>, hãy vào kênh để tham gia.\nNếu không thấy, liên hệ <@{}>",
                            true_channel_id, host_id
                        );
                        cmd.create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content(msg)
                                    .ephemeral(true),
                            ),
                        )
                        .await?;
                    }
                },
                Err(_) => {
                    cmd.create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("❌ Lỗi kết nối tới phòng chơi.")
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
