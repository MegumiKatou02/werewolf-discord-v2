use std::sync::Arc;

use anyhow::Result;
use serenity::all::{
    ButtonStyle, ChannelId, CreateActionRow, CreateAttachment, CreateButton, CreateEmbed,
    CreateMessage, MessageId, UserId,
};
use tokio::task::JoinSet;

use crate::{
    game::{canvas::create_avatar_collage, room::GameRoom},
    types::player::PlayerInfo,
};

pub async fn execute_vote_phase(room: &mut GameRoom) -> Result<()> {
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
    let shared_image = Arc::new(image_data);

    let embed_template = CreateEmbed::new()
        .title("📋 Danh sách người chơi")
        .color(0x00ae86)
        .image("attachment://players.png");

    let http = room.http.clone();
    let mut set = JoinSet::new();

    for player in room.players.iter() {
        let user_id = player.user_id;
        let is_alive = player.alive;
        let can_vote = player.can_vote;

        let (prompt, components) = build_vote_prompt_and_components(user_id, is_alive, can_vote)?;

        let http = http.clone();
        let embed = embed_template.clone();
        let image = shared_image.clone();

        set.spawn(async move {
            let dm = user_id.create_dm_channel(&http).await?;

            let attachment = CreateAttachment::bytes(image.as_ref().clone(), "players.png");

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
                room.vote_messages
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

fn build_vote_prompt_and_components(
    owner_id: UserId,
    is_alive: bool,
    can_vote: bool,
) -> Result<(String, Vec<CreateActionRow>)> {
    let make_row = |btns: Vec<CreateButton>| vec![CreateActionRow::Buttons(btns)];

    // TRƯỜNG HỢP 1: ĐÃ CHẾT
    if !is_alive {
        return Ok((
            "💀 **Bạn đã chết!** Hãy theo dõi những người còn lại tranh luận.".to_string(),
            vec![],
        ));
    }

    // TRƯỜNG HỢP 2: CÒN SỐNG
    let mut prompt =
        "⚖️ **Đã đến giờ phán quyết!**\nHãy bấm nút bên dưới để chọn người treo cổ.".to_string();

    let mut vote_btn = CreateButton::new(format!("vote_execution_req_{}", owner_id))
        .label("🗳️ Bỏ phiếu Treo cổ")
        // .emoji("🗳️")
        .style(ButtonStyle::Secondary);

    if !can_vote {
        prompt = "🤐 **Bạn bị cấm bỏ phiếu trong lượt này!**".to_string();
        vote_btn = vote_btn.disabled(true).label("Bị cấm vote");
    }

    Ok((prompt, make_row(vec![vote_btn])))
}
