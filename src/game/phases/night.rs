use anyhow::Result;
use serenity::all::{
    ButtonStyle, ChannelId, CreateActionRow, CreateAttachment, CreateButton, CreateEmbed,
    CreateMessage, MessageId, UserId,
};
use tokio::task::JoinSet;

use crate::game::canvas::create_avatar_collage;
use crate::game::room::GameRoom;
use crate::types::player::PlayerInfo;
use crate::types::Faction;
use crate::utils::response::row_single;
use crate::utils::role::RoleId;

pub async fn execute_night_phase(room: &mut GameRoom) -> Result<()> {
    // có vẻ đúng
    // room.night_messages.clear();

    let night_num = room.game_state.night_count;
    let night_title = if night_num == 1 {
        "đầu tiên".to_string()
    } else {
        format!("thứ {}", night_num)
    };

    let canvas_players: Vec<PlayerInfo> = room
        .players
        .iter()
        .map(|p| PlayerInfo {
            user_id: p.user_id.get(),
            username: p.name.clone(),
            avatar_url: p.avatar_url.clone(),
            global_name: None,
            alive: p.alive,
        })
        .collect();

    let image_data = create_avatar_collage(&canvas_players).await?;

    let embed_template = CreateEmbed::new()
        .title("📋 Danh sách người chơi")
        .color(0x00ae86)
        .image("attachment://players.png");

    let http = room.http.clone();
    let night_title = night_title.to_string();

    let mut set = JoinSet::new();

    for player in room.players.iter() {
        let role_id = player.role.id();
        let faction = player.role.faction();
        let user_id = player.user_id;
        let can_use_skill = player.can_use_skill;

        let build_result = build_night_prompt_and_components(room, user_id, role_id, can_use_skill);

        if let Ok((prompt, components)) = build_result {
            let http = http.clone();
            let image_data = image_data.clone();
            let embed_template = embed_template.clone();
            let night_title_clone = night_title.clone();

            set.spawn(async move {
                let dm = user_id.create_dm_channel(&http).await?;

                dm.send_message(
                    &http,
                    CreateMessage::new().content(format!("# 🌑 Đêm {}.", night_title_clone)),
                )
                .await?;

                let attachment = CreateAttachment::bytes(image_data, "players.png");

                let msg = dm
                    .send_message(
                        &http,
                        CreateMessage::new()
                            .content(prompt)
                            .add_embed(embed_template)
                            .add_file(attachment)
                            .components(components),
                    )
                    .await?;

                Ok::<(UserId, ChannelId, MessageId, Faction), serenity::Error>((
                    user_id,
                    msg.channel_id,
                    msg.id,
                    faction,
                ))
            });
        }
    }

    while let Some(res) = set.join_next().await {
        match res {
            Ok(Ok((user_id, channel_id, message_id, faction))) => {
                room.night_messages
                    .entry(user_id)
                    .or_default()
                    .push((channel_id, message_id));

                if faction == Faction::Werewolf {
                    room.wolf_messages
                        .entry(user_id)
                        .or_default()
                        .push((channel_id, message_id));
                }
            }
            Ok(Err(e)) => {
                tracing::error!("Lỗi gửi tin nhắn đêm: {:?}", e);
            }
            Err(e) => {
                tracing::error!("Lỗi JoinError (Task bị panic): {:?}", e);
            }
        }
    }

    Ok(())
}

fn build_night_prompt_and_components(
    room: &GameRoom,
    owner_id: serenity::all::UserId,
    role_id: RoleId,
    can_use_skill: bool,
) -> Result<(String, Vec<CreateActionRow>)> {
    let row_two = |a: CreateButton, b: CreateButton| vec![CreateActionRow::Buttons(vec![a, b])];

    match role_id {
        RoleId::Villager => Ok((
            format!(
                "🌙 Bạn là dân làng, một đêm yên tĩnh trôi qua. Bạn hãy chờ {} giây cho đến sáng.",
                room.settings.night_time
            ),
            vec![],
        )),
        RoleId::Werewolf | RoleId::KittenWolf | RoleId::Voodoo | RoleId::Wolffluence => {
            let prompt = format!(
                "🌙 Bạn là **{}**. Hãy vote người cần giết trong {} giây.",
                role_id.name(),
                room.settings.wolf_vote_time
            );
            let components = row_single(
                format!("vote_target_wolf_{}", owner_id),
                "🗳️ Vote người cần giết",
                ButtonStyle::Secondary,
                false,
            );
            Ok((prompt, components))
        }
        RoleId::WolfSeer => {
            let prompt = "🌙 Bạn là **Sói Tiên Tri**. Bạn có thể xem ai có phải là Tiên Tri hay không."
                .to_string();
            let components = row_single(
                format!("view_target_wolfseer_{}", owner_id),
                "🔍 Xem vai trò",
                ButtonStyle::Secondary,
                !can_use_skill,
            );
            Ok((prompt, components))
        }
        RoleId::AlphaWerewolf => {
            let prompt =
                "🌙 Bạn là **Sói Trùm**. Bạn có thể che sói khỏi tiên tri (mỗi đêm 1 sói)."
                    .to_string();
            let components = row_single(
                format!("mask_target_alphawerewolf_{}", owner_id),
                "👤 Che sói",
                ButtonStyle::Secondary,
                !can_use_skill,
            );
            Ok((prompt, components))
        }

        RoleId::Bodyguard => {
            let prompt = "🌙 Bạn là **Bảo Vệ**. Hãy chọn người bạn muốn bảo vệ trong đêm nay."
                .to_string();
            let components = row_single(
                format!("protect_target_bodyguard_{}", owner_id),
                "🛡️ Bảo vệ người",
                ButtonStyle::Secondary,
                !can_use_skill,
            );
            Ok((prompt, components))
        }
        RoleId::Seer => {
            let prompt =
                "🌙 Bạn là **Tiên Tri**. Bạn có thể xem phe của một người chơi khác trong đêm nay."
                    .to_string();
            let components = row_single(
                format!("view_target_seer_{}", owner_id),
                "🔍 Xem phe",
                ButtonStyle::Secondary,
                !can_use_skill,
            );
            Ok((prompt, components))
        }
        RoleId::Detective => {
            let prompt = "🌙 Bạn là **Thám Tử**. Bạn có thể điều tra hai người chơi để biết họ cùng phe hay khác phe."
                .to_string();
            let components = row_single(
                format!("investigate_target_detective_{}", owner_id),
                "🔎 Điều tra người",
                ButtonStyle::Secondary,
                !can_use_skill,
            );
            Ok((prompt, components))
        }
        RoleId::Witch => {
            let (poison_count, heal_count, need_help) = room
                .players
                .iter()
                .find(|p| p.user_id == owner_id)
                .and_then(|p| p.role.as_any().downcast_ref::<crate::roles::Witch>())
                .map(|w| (w.poison_count, w.heal_count, w.need_help_person.is_some()))
                .unwrap_or((0, 0, false));

            let prompt = format!(
                "🌙 Bạn là **Phù Thuỷ**. (Bình độc: {}, Bình cứu: {}).",
                poison_count,
                heal_count
            );

            let mut poison = CreateButton::new(format!("poison_target_witch_{}", owner_id))
                    .label("💊 Đầu độc người")
                    .style(ButtonStyle::Secondary);
                if !can_use_skill || poison_count == 0 {
                    poison = poison.disabled(true);
                }

            let mut heal = CreateButton::new(format!("heal_target_witch_{}", owner_id))
                .label("💫 Cứu người")
                .style(ButtonStyle::Secondary);
            if !can_use_skill || heal_count == 0 || !need_help {
                heal = heal.disabled(true);
            }

            Ok((prompt, row_two(poison, heal)))
        }
        RoleId::Medium => {
            let revived_count = room
                .players
                .iter()
                .find(|p| p.user_id == owner_id)
                .and_then(|p| p.role.as_any().downcast_ref::<crate::roles::Medium>())
                .map(|m| m.revived_count)
                .unwrap_or(0);

            let prompt =
                "🌙 Bạn là **Thầy Đồng**. Bạn có thể hồi sinh một người phe dân đã chết (1 lần/ván)."
                    .to_string();
            let components = row_single(
                format!("revive_target_medium_{}", owner_id),
                "🔮 Hồi sinh người",
                ButtonStyle::Secondary,
                !can_use_skill || revived_count == 0,
            );
            Ok((prompt, components))
        }
        RoleId::FoxSpirit => {
            let have_skill = room
                .players
                .iter()
                .find(|p| p.user_id == owner_id)
                .and_then(|p| p.role.as_any().downcast_ref::<crate::roles::FoxSpirit>())
                .map(|f| f.is_have_skill)
                .unwrap_or(false);

            let prompt =
                "🦊 Bạn là **Cáo**. Mỗi đêm dậy soi 3 người tự chọn trong danh sách, nếu 1 trong 3 người đó là sói thì được báo \"Có sói\", nếu đoán hụt thì mất chức năng."
                    .to_string();
            let components = row_single(
                format!("view_target_foxspirit_{}", owner_id),
                "🔍 Tìm sói",
                ButtonStyle::Secondary,
                !can_use_skill || !have_skill,
            );
            Ok((prompt, components))
        }
        RoleId::Maid => {
            let prompt = "🌙 Bạn là **Hầu Gái**. Hãy chọn một người làm chủ của bạn (chỉ đêm đầu tiên)."
                .to_string();
            let disabled = !can_use_skill || room.game_state.night_count != 1;
            let components = row_single(
                format!("choose_master_maid_{}", owner_id),
                if disabled { "👑 Đã chọn chủ" } else { "👑 Chọn chủ" },
                ButtonStyle::Secondary,
                disabled,
            );
            Ok((prompt, components))
        }
        RoleId::Stalker => {
            let (stalk_count, kill_count) = room
                .players
                .iter()
                .find(|p| p.user_id == owner_id)
                .and_then(|p| p.role.as_any().downcast_ref::<crate::roles::Stalker>())
                .map(|s| (s.stalk_count, s.kill_count))
                .unwrap_or((0, 0));
            let prompt = format!(
                "👀 Bạn là **Stalker**. (Theo dõi: {}, Ám sát: {}).",
                stalk_count, kill_count
            );

            let mut stalk = CreateButton::new(format!("stalk_target_stalker_{}", owner_id))
                .label("👀 Theo dõi")
                .style(ButtonStyle::Secondary);
            if !can_use_skill || stalk_count == 0 {
                stalk = stalk.disabled(true);
            }

            let mut kill = CreateButton::new(format!("kill_target_stalker_{}", owner_id))
                .label("🔪 Ám sát")
                .style(ButtonStyle::Secondary);
            if !can_use_skill || kill_count == 0 {
                kill = kill.disabled(true);
            }

            Ok((prompt, row_two(stalk, kill)))
        }
        RoleId::Puppeteer => {
            let target_count = room
                .players
                .iter()
                .find(|p| p.user_id == owner_id)
                .and_then(|p| p.role.as_any().downcast_ref::<crate::roles::Puppeteer>())
                .map(|p| p.target_count)
                .unwrap_or(0);
            let prompt =
                "🎭 Bạn là **Người Múa Rối**. Một lần duy nhất, bạn có thể chỉ định Sói ăn thịt một người."
                    .to_string();
            let components = row_single(
                format!("puppet_target_puppeteer_{}", owner_id),
                if target_count == 0 {
                    "🎭 Đã chỉ định mục tiêu"
                } else {
                    "🎭 Chỉ định mục tiêu"
                },
                ButtonStyle::Secondary,
                !can_use_skill || target_count == 0,
            );
            Ok((prompt, components))
        }

        RoleId::Dead => Ok((
            "💀 Bạn đã bị chết, hãy trò chuyện với hội người âm của bạn.".to_string(),
            vec![],
        )),
        RoleId::Fool => Ok((
            "⚜️ Bạn là thằng ngố, nhiệm vụ của bạn là lừa những người khác vote bạn để chiến thắng."
                .to_string(),
            vec![],
        )),
        RoleId::Lycan | RoleId::Elder | RoleId::Gunner | RoleId::Cursed | RoleId::Loudmouth => {
            Ok((format!("🌙 Bạn là **{}**.", role_id.name()), vec![]))
        }
    }
}
