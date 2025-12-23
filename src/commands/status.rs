use crate::bot::BotData;
use crate::commands::{CommandFuture, SlashCommand};
use crate::game::{RoomEvent, RoomStatus};
use serenity::all::*;
use std::sync::Arc;
use tokio::sync::oneshot;

pub struct StatusCommand;

impl SlashCommand for StatusCommand {
    fn name(&self) -> &'static str {
        "status"
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
                registry.get(&guild_id).cloned()
            };

            let room_handle = match room_handle {
                Some(h) => h,
                None => {
                    let embed = CreateEmbed::new()
                        .color(0x95a5a6)
                        .title("🎮 TRẠNG THÁI PHÒNG MA SÓI")
                        .description("```⚠️ Hiện không có phòng Ma Sói nào trong server!```")
                        .field("💡 Hướng Dẫn", "> Sử dụng lệnh `/masoi-create` để tạo phòng mới\n> Sử dụng `/huongdan` để xem hướng dẫn chi tiết", false)
                        .footer(CreateEmbedFooter::new("Hẹ hẹ hẹ"))
                        .timestamp(Timestamp::now());

                    cmd.create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new().add_embed(embed),
                        ),
                    )
                    .await?;
                    return Ok(());
                }
            };

            let (tx, rx) = oneshot::channel();
            if room_handle
                .sender
                .send(RoomEvent::StatusRequest { reply: tx })
                .is_err()
            {
                cmd.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("❌ Phòng chơi không phản hồi (có thể đã đóng).")
                            .ephemeral(true),
                    ),
                )
                .await?;
                return Ok(());
            }

            let snapshot = match rx.await {
                Ok(s) => s,
                Err(_) => return Ok(()),
            };

            let (color, icon, status_text) = match snapshot.status {
                RoomStatus::Waiting => (
                    0x3498db,
                    "⌛",
                    "```ini\n[Phòng đang chờ người chơi tham gia...]\n```",
                ),
                RoomStatus::Starting => (0xe74c3c, "🎯", "```fix\n[Trò chơi đang diễn ra...]\n```"),
                RoomStatus::Ended => (0x2ecc71, "🏁", "```diff\n+ Trò chơi đã kết thúc\n```"),
            };

            let title_suffix = if snapshot.status == RoomStatus::Starting {
                format!(" #{}", snapshot.game_state.night_count)
            } else {
                "".to_string()
            };

            let mut embed = CreateEmbed::new()
                .color(color)
                .title(format!("{} PHÒNG MA SÓI{}", icon, title_suffix))
                .description(status_text)
                .field("👑 Chủ Phòng", format!("> <@{}>", snapshot.host_id), true)
                .field(
                    "👥 Số Người Chơi",
                    format!("> {}/18", snapshot.players.len()),
                    true,
                );

            if snapshot.status == RoomStatus::Starting {
                let alive_count = snapshot.players.iter().filter(|p| p.alive).count();
                let dead_count = snapshot.players.len() - alive_count;

                let (phase_icon, phase_text) = match snapshot.game_state.phase {
                    crate::game::state::Phase::Night => ("🌙", "Ban Đêm"),
                    crate::game::state::Phase::Day => ("☀️", "Ban Ngày"),
                    crate::game::state::Phase::Voting => ("🗳️", "Bỏ Phiếu"),
                    _ => ("❓", "Khác"),
                };

                embed = embed
                    .field(
                        format!("{} Phase Hiện Tại", phase_icon),
                        format!("> {}", phase_text),
                        true,
                    )
                    .field("❤️ Còn Sống", format!("> {}", alive_count), true)
                    .field("💀 Đã Chết", format!("> {}", dead_count), true);
            }

            let player_list_str = if snapshot.players.is_empty() {
                "> *Chưa có người chơi nào tham gia*".to_string()
            } else {
                snapshot
                    .players
                    .iter()
                    .enumerate()
                    .map(|(index, p)| {
                        let is_host = p.user_id == snapshot.host_id;
                        let status_icon = if p.alive { "🟢" } else { "💀" };
                        let number = format!("{:02}", index + 1);
                        let crown = if is_host { " 👑" } else { "" };

                        format!("`{}` {} **{}**{}", number, status_icon, p.name, crown)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };

            embed = embed.field("📋 Danh Sách Người Chơi", player_list_str, false);

            let footer_text = match snapshot.status {
                RoomStatus::Waiting => "💡 Sử dụng /masoi-join để tham gia phòng",
                RoomStatus::Starting => "🎲 Game đang diễn ra, hãy đợi ván sau để tham gia",
                RoomStatus::Ended => "🔄 Sử dụng /masoi-create để tạo phòng mới",
            };

            embed = embed
                .footer(CreateEmbedFooter::new(footer_text))
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
