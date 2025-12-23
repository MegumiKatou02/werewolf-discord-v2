use anyhow::Result;
use serenity::all::{
    ButtonStyle, ChannelId, CreateActionRow, CreateAttachment, CreateButton, CreateEmbed,
    CreateMessage, MessageId, UserId,
};
use tokio::task::JoinSet;

use crate::{
    game::{canvas::create_avatar_collage, room::GameRoom},
    roles::{Gunner, VoodooWerewolf},
    types::player::PlayerInfo,
    utils::role::RoleId,
};

pub async fn execute_day_phase(room: &mut GameRoom) -> Result<()> {
    room.night_messages.clear();

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
    let mut set = JoinSet::new();

    for player in room.players.iter() {
        let user_id = player.user_id;
        let role_id = player.role.id();
        let can_use_skill = player.can_use_skill;

        let (prompt, components) = build_day_prompt_and_components(
            room,
            user_id,
            role_id,
            can_use_skill,
            room.game_state.night_count,
        )?;

        let http = http.clone();
        let embed = embed_template.clone();
        let image_data = image_data.clone();

        set.spawn(async move {
            let dm = user_id.create_dm_channel(&http).await?;

            let attachment = CreateAttachment::bytes(image_data, "players.png");

            let msg = dm
                .send_message(
                    &http,
                    CreateMessage::new()
                        .content(prompt)
                        .add_embed(embed)
                        .add_file(attachment)
                        .components(components),
                )
                .await?;

            Ok::<(UserId, ChannelId, MessageId), serenity::Error>((user_id, msg.channel_id, msg.id))
        });
    }

    while let Some(res) = set.join_next().await {
        match res {
            Ok(Ok((user_id, channel_id, message_id))) => {
                room.day_messages
                    .entry(user_id)
                    .or_default()
                    .push((channel_id, message_id));
            }
            Err(e) => tracing::error!("Lỗi task: {:?}", e),
            _ => {}
        }
    }

    Ok(())
}

fn build_day_prompt_and_components(
    room: &GameRoom,
    owner_id: UserId,
    role_id: RoleId,
    can_use_skill: bool,
    night_count: i32,
) -> Result<(String, Vec<CreateActionRow>)> {
    let make_row = |btns: Vec<CreateButton>| vec![CreateActionRow::Buttons(btns)];

    match role_id {
        RoleId::Gunner => {
            let bullets = room
                .players
                .iter()
                .find(|p| p.user_id == owner_id)
                .and_then(|p| p.role.as_any().downcast_ref::<Gunner>())
                .map(|g| g.bullets)
                .unwrap_or(0);

            let prompt = format!("🔫 Bạn là Xạ Thủ. Bạn còn **{}** viên đạn.", bullets);

            let mut shot_btn = CreateButton::new(format!("gunner_shoot_{}", owner_id))
                .label("Bắn người")
                .emoji('🔫')
                .style(ButtonStyle::Danger);

            if !can_use_skill || bullets <= 0 || night_count == 1 {
                shot_btn = shot_btn.disabled(true);
            }

            Ok((prompt, make_row(vec![shot_btn])))
        }

        RoleId::Voodoo => {
            let voodoo_count = room
                .players
                .iter()
                .find(|p| p.user_id == owner_id)
                .and_then(|p| p.role.as_any().downcast_ref::<VoodooWerewolf>())
                .map(|g| g.voodoo_count)
                .unwrap_or(0);

            let prompt = format!(
                "Bạn là Sói Tà Thuật. Bạn có thể cho người chơi khác ác mộng **{}** lần.",
                voodoo_count
            );

            let mut voodoo_btn = CreateButton::new(format!("voodoo_voodoo_{}", owner_id))
                .label("Ác mộng")
                .emoji('🌘')
                .style(ButtonStyle::Secondary);

            if !can_use_skill || voodoo_count <= 0 {
                voodoo_btn = voodoo_btn.disabled(true);
            }

            Ok((prompt, make_row(vec![voodoo_btn])))
        }

        _ => {
            let prompt = "Hãy tham gia biện luận và bỏ phiếu.".to_string();
            Ok((prompt, vec![]))
        }
    }
}
