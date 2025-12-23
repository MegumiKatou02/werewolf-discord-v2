use chrono::Utc;
use serenity::all::{
    ChannelId, ComponentInteraction, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption, MessageId, UserId,
};
use serenity::builder::EditInteractionResponse;
use serenity::{
    all::{
        ActionRowComponent, ComponentInteractionDataKind, CreateActionRow, CreateInputText,
        CreateInteractionResponse, CreateInteractionResponseMessage, CreateModal, GuildId,
        InputTextStyle, Interaction, Message, Ready,
    },
    async_trait,
    prelude::*,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::oneshot;

use crate::constants::MAX_FILE_SIZE;
use crate::game::room::RoomHandle;
use crate::game::{ChatFile, MessageTypeStore};
use crate::types::Player;
use crate::{
    bot::BotData,
    commands::{all_commands, role::get_role_menu_row},
    game::{RoomEvent, RoomSnapshot, StartGameResult},
    types::types::InteractionWrapper,
    utils::{
        embed::create_werewolf_embed,
        role::convert_faction_role,
        role_parser::{parse_roles_from_json_string, parse_roles_from_string},
    },
};
use crate::{client::handler::command_handler, commands::guide::get_guide_content};

pub struct Handler {
    pub data: Arc<BotData>,
}

impl Handler {
    async fn handle_target_selection_menu<F>(
        &self,
        ctx: &Context,
        component: &ComponentInteraction,
        trigger_prefix: &str,
        menu_id: &str,
        placeholder: &str,
        filter_predicate: F,
    ) -> Option<(ChannelId, MessageId)>
    where
        F: Fn(&Player) -> bool,
    {
        let custom_id = &component.data.custom_id;

        let owner_id_str = custom_id.strip_prefix(trigger_prefix).unwrap_or("");
        if component.user.id.to_string() != owner_id_str {
            self.reply_error(ctx, component, "❌ Nút này không phải của bạn!")
                .await;
            return None;
        }

        let room_handle = match self.get_room_handle_by_user(component.user.id).await {
            Some(h) => h,
            None => {
                self.reply_error(
                    ctx,
                    component,
                    "❌ Bạn không trong ván game nào/Phòng đã xóa.",
                )
                .await;
                return None;
            }
        };

        let (tx, rx) = tokio::sync::oneshot::channel();
        if room_handle
            .sender
            .send(RoomEvent::GetAllPlayers { reply: tx })
            .is_err()
        {
            return None;
        }
        let all_players = match rx.await {
            Ok(p) => p,
            Err(_) => return None,
        };

        let mut options = Vec::new();
        for (index, player) in all_players
            .iter()
            .filter(|p| filter_predicate(p))
            .enumerate()
        {
            let label = format!("{}. {}", index + 1, player.name);
            options
                .push(CreateSelectMenuOption::new(label, player.user_id.to_string()).emoji('👤'));
        }

        options.push(CreateSelectMenuOption::new("Hủy bỏ", "cancel_action").emoji('❌'));
        if options.len() > 25 {
            options.truncate(25);
        }

        let select_menu = CreateSelectMenu::new(menu_id, CreateSelectMenuKind::String { options })
            .placeholder(placeholder);

        if let Err(e) = component
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("Hãy chọn mục tiêu:")
                        .select_menu(select_menu), // .ephemeral(true),
                ),
            )
            .await
        {
            tracing::error!("Lỗi gửi menu: {:?}", e);
            return None;
        }

        match component.get_response(&ctx.http).await {
            Ok(msg) => Some((msg.channel_id, msg.id)),
            Err(e) => {
                tracing::error!("Không lấy được response message: {:?}", e);
                None
            }
        }
    }

    async fn get_room_handle_by_user(&self, user_id: UserId) -> Option<RoomHandle> {
        let p_reg = self.data.player_registry.read().await;
        let guild_id = *p_reg.get(&user_id)?;
        drop(p_reg);

        let r_reg = self.data.room_registry.read().await;
        r_reg.get(&guild_id).cloned()
    }

    async fn reply_error(&self, ctx: &Context, component: &ComponentInteraction, msg: &str) {
        let _ = component
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(msg)
                        .ephemeral(true),
                ),
            )
            .await;
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(cmd) => {
                for command in all_commands() {
                    if command.name() == cmd.data.name {
                        if let Err(e) = command.run(ctx, cmd, self.data.clone()).await {
                            tracing::error!("{:?}", e);
                        }
                        return;
                    }
                }
            }
            Interaction::Component(component) => {
                let custom_id = &component.data.custom_id;
                println!("DEBUG: Nhận được Component ID: {}", custom_id);

                if custom_id.starts_with("vote_target_wolf_") {
                    let msg_info = self
                        .handle_target_selection_menu(
                            &ctx,
                            &component,
                            "vote_target_wolf_",
                            "wolf_submit_vote",
                            "💀 Chọn nạn nhân...",
                            |p| p.alive && !p.is_werewolf(),
                        )
                        .await;

                    if let Some((channel_id, message_id)) = msg_info {
                        if let Some(room_handle) =
                            self.get_room_handle_by_user(component.user.id).await
                        {
                            let event = RoomEvent::RegisterInteraction {
                                user_id: component.user.id,
                                channel_id,
                                message_id,
                                message_type_store: MessageTypeStore::WolfMessage,
                            };

                            let _ = room_handle.sender.send(event);
                        }
                    }
                    return;
                }
                if custom_id == "wolf_submit_vote" {
                    let values = match &component.data.kind {
                        ComponentInteractionDataKind::StringSelect { values } => values,
                        _ => return,
                    };

                    if values.is_empty() {
                        return;
                    }

                    let target_id: UserId;

                    if let Some(target_id_str) = values.first() {
                        let id = target_id_str.parse::<u64>().unwrap();
                        target_id = UserId::new(id);
                    } else {
                        return;
                    }

                    let room_handle = match self.get_room_handle_by_user(component.user.id).await {
                        Some(h) => h,
                        None => {
                            self.reply_error(&ctx, &component, "❌ Lỗi: Không tìm thấy phòng.")
                                .await;
                            return;
                        }
                    };

                    let event = RoomEvent::WolfVote {
                        user_id: component.user.id,
                        target: target_id,
                    };

                    if let Err(_) = room_handle.sender.send(event) {
                        self.reply_error(&ctx, &component, "❌ Lỗi: Game đã kết thúc.")
                            .await;
                        return;
                    }

                    println!("target_id {}", target_id);
                    let _ = component
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::UpdateMessage(
                                CreateInteractionResponseMessage::new()
                                    .content(format!(
                                        "✅ Đã ghi nhận vote của bạn cho <@{}>!",
                                        target_id
                                    ))
                                    .components(vec![])
                                    .ephemeral(true),
                            ),
                        )
                        .await;

                    return;
                }

                if custom_id.starts_with("protect_target_bodyguard_") {
                    let msg_info = self
                        .handle_target_selection_menu(
                            &ctx,
                            &component,
                            "protect_target_bodyguard_",
                            "bodyguard_submit_protect",
                            "🛡️ Chọn người cần bảo vệ...",
                            |p| p.alive && p.user_id != component.user.id,
                        )
                        .await;

                    if let Some((channel_id, message_id)) = msg_info {
                        if let Some(room_handle) =
                            self.get_room_handle_by_user(component.user.id).await
                        {
                            let event = RoomEvent::RegisterInteraction {
                                user_id: component.user.id,
                                channel_id,
                                message_id,
                                message_type_store: MessageTypeStore::NightMessage,
                            };

                            let _ = room_handle.sender.send(event);
                        }
                    }
                    return;
                }

                if custom_id == "bodyguard_submit_protect" {
                    let values = match &component.data.kind {
                        ComponentInteractionDataKind::StringSelect { values } => values,
                        _ => return,
                    };

                    if values.is_empty() {
                        return;
                    }

                    let first_value = &values[0];
                    if first_value == "cancel_action" {
                        let _ = component
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::UpdateMessage(
                                    CreateInteractionResponseMessage::new()
                                        .content("❌ Đã hủy bỏ hành động bảo vệ.")
                                        .components(vec![]),
                                ),
                            )
                            .await;
                        return;
                    }

                    let target_id: UserId = match first_value.parse::<u64>() {
                        Ok(id) => UserId::new(id),
                        Err(_) => return,
                    };

                    let room_handle = match self.get_room_handle_by_user(component.user.id).await {
                        Some(h) => h,
                        None => {
                            self.reply_error(&ctx, &component, "❌ Lỗi: Không tìm thấy phòng.")
                                .await;
                            return;
                        }
                    };

                    let event = RoomEvent::BodyguardProtect {
                        user_id: component.user.id,
                        target: target_id,
                    };

                    if let Err(_) = room_handle.sender.send(event) {
                        self.reply_error(&ctx, &component, "❌ Lỗi: Game đã kết thúc.")
                            .await;
                        return;
                    }

                    let _ = component
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::UpdateMessage(
                                CreateInteractionResponseMessage::new()
                                    .content(format!("✅ 🛡️ Bạn đã bảo vệ <@{}>!", target_id))
                                    .components(vec![]),
                            ),
                        )
                        .await;

                    return;
                }

                // === SEER VIEW ===
                if custom_id.starts_with("view_target_seer_") {
                    let msg_info = self
                        .handle_target_selection_menu(
                            &ctx,
                            &component,
                            "view_target_seer_",
                            "seer_submit_view",
                            "👁️ Chọn người cần xem phe...",
                            |p| p.alive && p.user_id != component.user.id,
                        )
                        .await;

                    if let Some((channel_id, message_id)) = msg_info {
                        if let Some(room_handle) =
                            self.get_room_handle_by_user(component.user.id).await
                        {
                            let event = RoomEvent::RegisterInteraction {
                                user_id: component.user.id,
                                channel_id,
                                message_id,
                                message_type_store: MessageTypeStore::NightMessage,
                            };

                            let _ = room_handle.sender.send(event);
                        }
                    }
                    return;
                }

                if custom_id == "seer_submit_view" {
                    let values = match &component.data.kind {
                        ComponentInteractionDataKind::StringSelect { values } => values,
                        _ => return,
                    };

                    if values.is_empty() {
                        return;
                    }

                    let first_value = &values[0];
                    if first_value == "cancel_action" {
                        let _ = component
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::UpdateMessage(
                                    CreateInteractionResponseMessage::new()
                                        .content("❌ Đã hủy bỏ hành động xem phe.")
                                        .components(vec![]),
                                ),
                            )
                            .await;
                        return;
                    }

                    let target_id: UserId = match first_value.parse::<u64>() {
                        Ok(id) => UserId::new(id),
                        Err(_) => return,
                    };

                    let room_handle = match self.get_room_handle_by_user(component.user.id).await {
                        Some(h) => h,
                        None => {
                            self.reply_error(&ctx, &component, "❌ Lỗi: Không tìm thấy phòng.")
                                .await;
                            return;
                        }
                    };

                    let event = RoomEvent::SeerView {
                        user_id: component.user.id,
                        target: target_id,
                    };

                    if let Err(_) = room_handle.sender.send(event) {
                        self.reply_error(&ctx, &component, "❌ Lỗi: Game đã kết thúc.")
                            .await;
                        return;
                    }

                    let _ = component
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::UpdateMessage(
                                CreateInteractionResponseMessage::new()
                                    .content("✅ Đang xem phe...")
                                    .components(vec![]),
                            ),
                        )
                        .await;

                    return;
                }

                if custom_id.starts_with("guide_select:") {
                    let owner_id = custom_id.split(":").last().unwrap_or("");
                    if component.user.id.to_string() != owner_id {
                        let _ = component
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new()
                                        .content("❌ Chỉ người gọi lệnh mới có thể chuyển trang!")
                                        .ephemeral(true),
                                ),
                            )
                            .await;
                        return;
                    }

                    let now = Utc::now().timestamp();
                    let msg_time = component.message.timestamp.unix_timestamp();

                    if now - msg_time > 300 {
                        let _ = component
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new()
                                        .content("⚠️ Menu đã hết hạn. Vui lòng gõ lại /huongdan")
                                        .ephemeral(true),
                                ),
                            )
                            .await;
                        return;
                    }

                    if let ComponentInteractionDataKind::StringSelect { values } =
                        &component.data.kind
                    {
                        let selected_value = &values[0];
                        let (new_embed, new_row) = get_guide_content(selected_value, owner_id);

                        let _ = component
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::UpdateMessage(
                                    CreateInteractionResponseMessage::new()
                                        .add_embed(new_embed)
                                        .components(vec![new_row]),
                                ),
                            )
                            .await;
                    }
                }
                if custom_id.starts_with("role_select:") {
                    let owner_id = custom_id.split(":").last().unwrap_or("");

                    if component.user.id.to_string() != owner_id {
                        return;
                    }

                    if let ComponentInteractionDataKind::StringSelect { values } =
                        &component.data.kind
                    {
                        let selected_id = &values[0];

                        if let Some(role) = self.data.roles_json.get(selected_id) {
                            let file_name =
                                format!("{}.png", role.e_name.to_lowercase().replace(" ", "_"));
                            let faction_name = convert_faction_role(role.faction);

                            let description =
                                format!("{}\n\n**Phe:** {}", role.description, faction_name);
                            let data_embed = create_werewolf_embed(
                                &file_name,
                                &format!("{} ({})", role.title, role.e_name),
                                &description,
                            )
                            .await
                            .unwrap();

                            let row = get_role_menu_row(owner_id, &self.data.roles_json);

                            let _ = component
                                .create_response(
                                    &ctx.http,
                                    CreateInteractionResponse::UpdateMessage(
                                        CreateInteractionResponseMessage::new()
                                            .add_embed(data_embed.embed)
                                            .add_file(data_embed.attachment)
                                            .components(vec![row]),
                                    ),
                                )
                                .await;
                        }
                    }
                }

                let guild_id = match component.guild_id {
                    Some(id) => id,
                    None => return,
                };

                if custom_id == "start_default" {
                    if let Err(e) = component.defer_ephemeral(&ctx.http).await {
                        tracing::error!("Lỗi defer: {:?}", e);
                        return;
                    }
                    if let Err(e) = handle_start_game(
                        &ctx,
                        InteractionWrapper::Component(component),
                        &self.data,
                        guild_id,
                        None,
                    )
                    .await
                    {
                        tracing::error!("start_game error: {:?}", e);
                    }
                } else if custom_id == "start_custom_json" {
                    let modal = CreateModal::new("modal_start_json", "Tuỳ chỉnh vai trò (JSON)")
                        .components(vec![CreateActionRow::InputText(
                            CreateInputText::new(
                                InputTextStyle::Paragraph,
                                "JSON Data",
                                "roles_json",
                            )
                            .placeholder("{\"0\": 2, \"1\": 3}")
                            .required(true),
                        )]);
                    let _ = component
                        .create_response(&ctx.http, CreateInteractionResponse::Modal(modal))
                        .await;
                } else if custom_id == "start_custom_name" {
                    let modal = CreateModal::new("modal_start_name", "Tuỳ chỉnh vai trò (Tên)")
                        .components(vec![CreateActionRow::InputText(
                            CreateInputText::new(
                                InputTextStyle::Paragraph,
                                "Danh sách",
                                "roles_names",
                            )
                            .placeholder("masoi: 2, danlang: 3")
                            .required(true),
                        )]);
                    let _ = component
                        .create_response(&ctx.http, CreateInteractionResponse::Modal(modal))
                        .await;
                }
            }
            Interaction::Modal(modal) => {
                let custom_id = modal.data.custom_id.clone();
                let guild_id = match modal.guild_id {
                    Some(id) => id,
                    None => return,
                };

                if custom_id == "modal_start_name" {
                    let input = match modal
                        .data
                        .components
                        .first()
                        .and_then(|r| r.components.first())
                    {
                        Some(ActionRowComponent::InputText(text)) => &text.value,
                        _ => return,
                    };

                    if let Err(e) = modal.defer_ephemeral(&ctx.http).await {
                        tracing::error!("Lỗi defer modal: {:?}", e);
                        return;
                    }

                    let input_str = input.as_deref().unwrap_or("");
                    match parse_roles_from_string(input_str) {
                        Ok(roles) => {
                            if let Err(e) = handle_start_game(
                                &ctx,
                                InteractionWrapper::Modal(modal),
                                &self.data,
                                guild_id,
                                Some(roles),
                            )
                            .await
                            {
                                tracing::error!("start_game error: {:?}", e);
                            }
                        }
                        Err(e) => {
                            let _ = modal
                                .edit_response(
                                    &ctx.http,
                                    EditInteractionResponse::new()
                                        .content(format!("❌ Lỗi: {}", e)),
                                )
                                .await;
                        }
                    }
                    return;
                } else if custom_id == "modal_start_json" {
                    let input = match modal
                        .data
                        .components
                        .first()
                        .and_then(|r| r.components.first())
                    {
                        Some(ActionRowComponent::InputText(text)) => &text.value,
                        _ => return,
                    };

                    if let Err(e) = modal.defer_ephemeral(&ctx.http).await {
                        tracing::error!("Lỗi defer modal: {:?}", e);
                        return;
                    }

                    let input_str = input.as_deref().unwrap_or("");
                    match parse_roles_from_json_string(input_str) {
                        Ok(roles) => {
                            if let Err(e) = handle_start_game(
                                &ctx,
                                InteractionWrapper::Modal(modal),
                                &self.data,
                                guild_id,
                                Some(roles),
                            )
                            .await
                            {
                                tracing::error!("start_game error: {:?}", e);
                            }
                        }
                        Err(e) => {
                            let _ = modal
                                .edit_response(
                                    &ctx.http,
                                    EditInteractionResponse::new().content(format!("❌ {}", e)),
                                )
                                .await;
                        }
                    }
                    return;
                }
            }
            _ => {}
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} đã sẵn sàng!", ready.user.name);
        tracing::info!("{} đã sẵn sàng!", ready.user.name);

        let guild_id = std::env::var("GUILD_ID")
            .map(|id| GuildId::new(id.parse::<u64>().expect("GUILD_ID phải là số")))
            .expect("Chưa cấu hình GUILD_ID trong .env");

        let mut commands_to_create = Vec::new();

        for cmd_obj in all_commands() {
            let builder =
                serenity::all::CreateCommand::new(cmd_obj.name()).description("Lệnh game Ma Sói");

            commands_to_create.push(builder);
        }

        if let Err(e) = guild_id.set_commands(&ctx.http, commands_to_create).await {
            tracing::error!("Lỗi khi đăng ký lệnh: {:?}", e);
        } else {
            tracing::info!("Đã đăng ký toàn bộ các lệnh thành công!");
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        command_handler(&ctx, &msg, &self.data).await;
        if msg.guild_id.is_none() {
            let target_guild_id = {
                let player_registry = self.data.player_registry.read().await;
                player_registry.get(&msg.author.id).copied()
            };
            if let Some(guild_id) = target_guild_id {
                let registry = self.data.room_registry.read().await;

                if let Some(handle) = registry.get(&guild_id) {
                    let _ = handle.sender.send(RoomEvent::WolfChat {
                        sender_id: msg.author.id,
                        sender_name: msg.author.name.clone(),
                        content: msg.content.clone(),
                    });

                    let attachments: Vec<ChatFile> = msg
                        .attachments
                        .iter()
                        .filter(|a| a.size <= MAX_FILE_SIZE)
                        .map(|a| ChatFile {
                            url: a.url.clone(),
                            filename: a.filename.clone(),
                            size: a.size,
                        })
                        .collect();

                    let _ = handle.sender.send(RoomEvent::DayChat {
                        sender_id: msg.author.id,
                        sender_name: msg.author.name.clone(),
                        content: msg.content.clone(),
                        attachments: attachments,
                    });

                    tracing::info!(
                        "Đã chuyển tin nhắn DM của {} vào phòng {}",
                        msg.author.name,
                        guild_id
                    );
                }
            }
        }
    }
}

fn parse_action_owner(custom_id: &str) -> Option<(&str, &str)> {
    let (action, owner) = custom_id.rsplit_once('_')?;

    if owner.chars().all(|c| c.is_numeric()) && !owner.is_empty() {
        return Some((action, owner));
    }

    None
}

fn parse_modal_owner(custom_id: &str) -> Option<(&str, String)> {
    //modal_wolf_vote:<ownerId>
    let mut it = custom_id.split(':');
    let kind = it.next()?;
    let owner = it.next()?.to_string();
    let key = match kind {
        "modal_wolf_vote" => "wolf_vote",
        "modal_seer_view" => "seer_view",
        "modal_bodyguard_protect" => "bodyguard_protect",
        "modal_witch_poison" => "witch_poison",
        "modal_witch_heal" => "witch_heal",
        _ => return None,
    };
    Some((key, owner))
}

async fn get_room_handle(
    data: &BotData,
    guild_id: GuildId,
) -> Result<crate::game::room::RoomHandle, String> {
    let registry = data.room_registry.read().await;
    registry
        .get(&guild_id)
        .cloned()
        .ok_or_else(|| "Không tìm thấy phòng chơi.".to_string())
}

async fn get_room_snapshot(handle: &crate::game::room::RoomHandle) -> Result<RoomSnapshot, String> {
    let (tx, rx) = oneshot::channel();
    let _ = handle.sender.send(RoomEvent::StatusRequest { reply: tx });
    rx.await.map_err(|_| "Phòng chơi đã đóng.".to_string())
}

fn parse_target_user_id(
    input: &str,
    snapshot: &RoomSnapshot,
) -> Result<serenity::all::UserId, String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("Bạn chưa nhập mục tiêu.".to_string());
    }

    if let Ok(idx) = s.parse::<usize>() {
        if idx == 0 || idx > snapshot.players.len() {
            return Err(format!(
                "Số thứ tự không hợp lệ (1..{}).",
                snapshot.players.len()
            ));
        }
        return Ok(snapshot.players[idx - 1].user_id);
    }

    // <@id> or <@!id>
    let cleaned = s
        .trim_start_matches("<@!")
        .trim_start_matches("<@")
        .trim_end_matches('>')
        .trim();

    let id_u64: u64 = cleaned
        .parse()
        .map_err(|_| "Mục tiêu không hợp lệ. Nhập số thứ tự hoặc UserId.".to_string())?;
    let uid = serenity::all::UserId::new(id_u64);

    if !snapshot.players.iter().any(|p| p.user_id == uid) {
        return Err("UserId không nằm trong phòng chơi.".to_string());
    }

    Ok(uid)
}

async fn handle_start_game(
    ctx: &Context,
    interaction: InteractionWrapper,
    data: &BotData,
    guild_id: GuildId,
    roles: Option<HashMap<u8, u8>>,
) -> anyhow::Result<()> {
    let user_id = interaction.user_id();

    let registry = data.room_registry.read().await;
    let handle = match registry.get(&guild_id) {
        Some(h) => h.clone(),
        None => {
            respond_error(ctx, interaction, "Không tìm thấy phòng chơi.".to_string()).await?;
            return Ok(());
        }
    };
    drop(registry);

    tracing::info!("ROLES: {:?}", roles);

    let (stx, srx) = oneshot::channel();
    let _ = handle.sender.send(RoomEvent::StatusRequest { reply: stx });
    let snapshot = match srx.await {
        Ok(s) => s,
        Err(_) => {
            respond_error(ctx, interaction, "Phòng chơi đã đóng.".to_string()).await?;
            return Ok(());
        }
    };

    if user_id != snapshot.host_id {
        respond_error(
            ctx,
            interaction,
            "❌ Chỉ Host của phòng mới có thể bắt đầu trò chơi.".to_string(),
        )
        .await?;
        return Ok(());
    }

    if let Some(ref map) = roles {
        let total: u32 = map.values().map(|v| *v as u32).sum();
        let players = snapshot.players.len() as u32;
        if total != players {
            respond_error(
                ctx,
                interaction,
                format!(
                    "Tổng số vai trò ({}) phải bằng số người chơi ({}).",
                    total, players
                ),
            )
            .await?;
            return Ok(());
        }

        let wolves = *map.get(&0).unwrap_or(&0) as u32;
        if wolves == 0 {
            respond_error(
                ctx,
                interaction,
                "Phải có ít nhất 1 Sói trong game.".to_string(),
            )
            .await?;
            return Ok(());
        }
    }

    let (tx, rx) = oneshot::channel();
    let _ = handle.sender.send(RoomEvent::StartGame {
        user_id,
        custom_roles: roles,
        reply: tx,
    });

    match rx.await {
        Ok(StartGameResult::Success) => {
            let name = snapshot
                .players
                .iter()
                .find(|p| p.user_id == user_id)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "Host".to_string());

            respond_success(
                ctx,
                interaction,
                format!("✅ {} đã bắt đầu trò chơi! Vai trò đã được chia.", name),
            )
            .await?;
        }
        Ok(StartGameResult::Error(msg)) => {
            respond_error(ctx, interaction, format!("❌ {}", msg)).await?;
        }
        Err(_) => {
            respond_error(ctx, interaction, "Phòng chơi đã đóng.".to_string()).await?;
        }
    }

    Ok(())
}

async fn respond_success(
    ctx: &Context,
    interaction: InteractionWrapper,
    content: String,
) -> serenity::Result<()> {
    let response = EditInteractionResponse::new()
        .content(content)
        .components(vec![]);

    match interaction {
        InteractionWrapper::Component(component) => component
            .edit_response(&ctx.http, response)
            .await
            .map(|_| ()),
        InteractionWrapper::Modal(modal) => {
            modal.edit_response(&ctx.http, response).await.map(|_| ())
        }
    }
}

async fn respond_error(
    ctx: &Context,
    interaction: InteractionWrapper,
    content: String,
) -> serenity::Result<()> {
    let response = EditInteractionResponse::new().content(format!("❌ {}", content));

    match interaction {
        InteractionWrapper::Component(component) => component
            .edit_response(&ctx.http, response)
            .await
            .map(|_| ()),
        InteractionWrapper::Modal(modal) => {
            modal.edit_response(&ctx.http, response).await.map(|_| ())
        }
    }
}
