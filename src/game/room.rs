use anyhow::Result;
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, SeedableRng};
use serenity::all::{
    ActionRowComponent, ChannelId, Context, CreateActionRow, CreateAttachment, CreateButton,
    CreateMessage, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, EditMessage,
    GuildId, Http, MessageId, UserId,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::task::JoinSet;
use tokio::time::{sleep, Duration};

use super::state::Phase;
use crate::game::phases::{
    execute_day_phase, execute_night_phase, execute_solve_phase, execute_vote_phase,
};
use crate::game::{
    GameState, JoinResult, LeaveResult, MessageTypeStore, RawFile, RoomEvent, RoomSettings,
    RoomSnapshot, RoomStatus, StartGameResult,
};
use crate::types::data::RolesData;
use crate::types::{Faction, Player};
use crate::utils::role::RoleId;

pub type RoomRegistry = Arc<RwLock<HashMap<GuildId, RoomHandle>>>;
pub type PlayerRegistry = Arc<RwLock<HashMap<UserId, GuildId>>>;

#[derive(Clone)]
pub struct RoomHandle {
    pub sender: mpsc::UnboundedSender<RoomEvent>,
}

pub struct GameRoom {
    pub(crate) guild_id: GuildId,
    pub(crate) host_id: UserId,
    pub(crate) channel_id: ChannelId,
    pub(crate) players: Vec<Player>,
    status: RoomStatus,
    pub(crate) game_state: GameState,
    pub(crate) settings: RoomSettings,
    pub(crate) roles_json: RolesData,
    pub(crate) http: Arc<Http>,
    pub(crate) http_client: reqwest::Client,

    pub(crate) night_messages:
        HashMap<UserId, Vec<(serenity::all::ChannelId, serenity::all::MessageId)>>,
    pub(crate) wolf_messages:
        HashMap<UserId, Vec<(serenity::all::ChannelId, serenity::all::MessageId)>>,
    pub(crate) day_messages:
        HashMap<UserId, Vec<(serenity::all::ChannelId, serenity::all::MessageId)>>,
    pub(crate) vote_messages:
        HashMap<UserId, Vec<(serenity::all::ChannelId, serenity::all::MessageId)>>,

    receiver: mpsc::UnboundedReceiver<RoomEvent>,

    sender: mpsc::UnboundedSender<RoomEvent>,

    phase_timer_cancel: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl GameRoom {
    pub fn new(
        guild_id: GuildId,
        host_id: UserId,
        channel_id: ChannelId,
        roles_json: RolesData,
        http: Arc<Http>,
    ) -> (Self, RoomHandle) {
        let (sender, receiver) = mpsc::unbounded_channel();

        let room = Self {
            guild_id,
            host_id,
            channel_id,
            http,
            http_client: reqwest::Client::new(),
            players: Vec::new(),
            status: RoomStatus::Waiting,
            game_state: GameState::new(),
            settings: RoomSettings::default(),
            roles_json,
            night_messages: HashMap::new(),
            wolf_messages: HashMap::new(),
            day_messages: HashMap::new(),
            vote_messages: HashMap::new(),
            receiver,
            sender: sender.clone(),
            phase_timer_cancel: Arc::new(Mutex::new(None)),
        };

        let handle = RoomHandle { sender };

        (room, handle)
    }

    pub async fn run(mut self) {
        tracing::info!("Room {} started", self.guild_id);

        while let Some(event) = self.receiver.recv().await {
            if let Err(e) = self.handle_event(event).await {
                tracing::error!("Error handling event in room {}: {:?}", self.guild_id, e);
            }

            if self.status == RoomStatus::Ended {
                break;
            }
        }

        self.cleanup().await;
        tracing::info!("Room {} ended", self.guild_id);
    }

    async fn handle_event(&mut self, event: RoomEvent) -> Result<()> {
        match event {
            RoomEvent::RegisterInteraction {
                user_id,
                channel_id,
                message_id,
                message_type_store,
            } => {
                tracing::info!("Đã đăng ký interaction cho user {}", user_id);
                match message_type_store {
                    MessageTypeStore::WolfMessage => {
                        self.wolf_messages
                            .entry(user_id)
                            .or_default()
                            .push((channel_id, message_id));
                    }
                    MessageTypeStore::NightMessage => {
                        self.night_messages
                            .entry(user_id)
                            .or_default()
                            .push((channel_id, message_id));
                    }
                    MessageTypeStore::DayMessage => {
                        self.day_messages
                            .entry(user_id)
                            .or_default()
                            .push((channel_id, message_id));
                    }
                    MessageTypeStore::VoteMessage => {
                        self.vote_messages
                            .entry(user_id)
                            .or_default()
                            .push((channel_id, message_id));
                    }
                }
            }
            RoomEvent::GetAllPlayers { reply } => {
                let _ = reply.send(self.players.clone());
            }
            RoomEvent::WolfChat {
                sender_id,
                sender_name,
                content,
            } => {
                if self.game_state.phase != Phase::Night {
                    return Ok(());
                }

                let is_valid = self
                    .players
                    .iter()
                    .any(|p| p.user_id == sender_id && p.is_werewolf() && p.alive);
                if !is_valid {
                    return Ok(());
                }

                for player in self.players.iter() {
                    if player.is_werewolf() && player.alive && player.user_id != sender_id {
                        let dm = player.user_id.create_dm_channel(&self.http).await?;
                        let msg = format!("**[🐺 {}]**: {}", sender_name, content);
                        let _ = dm.say(&self.http, msg).await;
                    }
                }
            }
            RoomEvent::DayChat {
                sender_id,
                sender_name,
                content,
                attachments,
            } => {
                // tracing::info!("DEBUG: Phase hiện tại là {:?}", self.game_state.phase);
                if !matches!(self.game_state.phase, Phase::Day | Phase::Voting) {
                    return Ok(());
                }

                let (is_sender_alive, can_sender_chat, sender_user_id) =
                    match self.players.iter().find(|p| p.user_id == sender_id) {
                        Some(p) => (p.alive, p.can_chat, p.user_id),
                        None => {
                            tracing::error!(
                                "DEBUG: Không tìm thấy người gửi {} trong danh sách players!",
                                sender_id
                            );
                            return Ok(());
                        }
                    };

                tracing::info!(
                    "DEBUG: Người gửi {} (Sống: {}, Chat: {})",
                    sender_name,
                    is_sender_alive,
                    can_sender_chat
                );

                if is_sender_alive && !can_sender_chat {
                    let http = self.http.clone();
                    tokio::spawn(async move {
                        if let Ok(dm) = sender_user_id.create_dm_channel(&http).await {
                            let _ = dm
                                .send_message(
                                    &http,
                                    CreateMessage::new()
                                        .content("⚠️ Bạn không thể chat trong hôm nay!"),
                                )
                                .await;
                        }
                    });
                    return Ok(());
                }

                let mut shared_files = Vec::new();
                if !attachments.is_empty() {
                    let mut download_set = JoinSet::new();
                    for file in attachments {
                        let client = self.http_client.clone();
                        download_set.spawn(async move {
                            match client.get(&file.url).send().await {
                                Ok(resp) => match resp.bytes().await {
                                    Ok(bytes) => Some(RawFile {
                                        data: Arc::new(bytes.to_vec()),
                                        name: file.filename,
                                    }),
                                    Err(_) => None,
                                },
                                Err(_) => None,
                            }
                        });
                    }

                    while let Some(res) = download_set.join_next().await {
                        if let Ok(Some(raw_file)) = res {
                            shared_files.push(raw_file);
                        }
                    }
                }
                let shared_files_arc = Arc::new(shared_files);

                let developer_id = UserId::new(604949724788817920);

                let formatted_content = if !is_sender_alive {
                    format!("_💀 **{}**: {}_", sender_name, content)
                } else if sender_id == developer_id {
                    format!("🔧 **{}**: {} (Dev)", sender_name, content)
                } else {
                    format!("🗣️ **{}**: {}", sender_name, content)
                };

                let http = self.http.clone();

                tracing::info!(
                    "DEBUG: Bắt đầu vòng lặp gửi tin cho {} người chơi...",
                    self.players.len()
                );

                for player in self.players.iter() {
                    if player.user_id == sender_id {
                        continue;
                    }

                    if !is_sender_alive && player.alive {
                        tracing::info!(
                            "DEBUG: Bỏ qua {} (Sống) vì người gửi đã Chết.",
                            player.name
                        );
                        continue;
                    }

                    tracing::info!("DEBUG: Đang chuẩn bị gửi cho {}...", player.name);

                    let recipient_id = player.user_id;
                    let http_clone = http.clone();
                    let content_clone = formatted_content.clone();
                    let files_ref = shared_files_arc.clone();

                    tokio::spawn(async move {
                        let dm = match recipient_id.create_dm_channel(&http_clone).await {
                            Ok(c) => c,
                            Err(e) => {
                                tracing::error!("Lỗi tạo DM: {:?}", e);
                                return;
                            }
                        };

                        let mut msg_builder = CreateMessage::new().content(content_clone);

                        for file in files_ref.iter() {
                            let attachment =
                                CreateAttachment::bytes(file.data.as_ref().clone(), &file.name);
                            msg_builder = msg_builder.add_file(attachment);
                        }

                        if let Err(e) = dm.send_message(&http_clone, msg_builder).await {
                            tracing::error!("Lỗi gửi tin: {:?}", e);
                        }
                    });
                }

                // while let Some(_) = broadcast_set.join_next().await {}
            }
            RoomEvent::JoinRequest {
                user_id,
                name,
                avatar_url,
                channel_id,
                reply,
            } => {
                if self.status != RoomStatus::Waiting {
                    let _ = reply.send(JoinResult::GameStarted);
                    return Ok(());
                }

                if self.players.iter().any(|p| p.user_id == user_id) {
                    let _ = reply.send(JoinResult::AlreadyJoined);
                    return Ok(());
                }

                if self.players.len() >= 18 {
                    let _ = reply.send(JoinResult::RoomFull);
                    return Ok(());
                }

                if self.channel_id != channel_id {
                    let _ = reply.send(JoinResult::WrongChannel(self.channel_id, self.host_id));
                    return Ok(());
                }

                let role = crate::roles::create_role(crate::utils::role::RoleId::Villager);
                self.players
                    .push(Player::new(user_id, name, role, avatar_url));

                let _ = reply.send(JoinResult::Success(self.players.len()));
            }
            RoomEvent::LeaveRequest { user_id, reply } => {
                if self.status != RoomStatus::Waiting {
                    let _ = reply.send(LeaveResult::GameStarted);
                    return Ok(());
                }

                if let Some(index) = self.players.iter().position(|p| p.user_id == user_id) {
                    self.players.remove(index);

                    if self.players.is_empty() {
                        self.status = RoomStatus::Ended;
                        let _ = reply.send(LeaveResult::RoomEmpty);
                    } else {
                        let _ = reply.send(LeaveResult::Success(self.players.len()));
                    }
                } else {
                    let _ = reply.send(LeaveResult::NotJoined);
                }
            }
            RoomEvent::StatusRequest { reply } => {
                let snapshot = RoomSnapshot {
                    status: self.status.clone(),
                    host_id: self.host_id,
                    players: self.players.clone(),
                    game_state: self.game_state.clone(),
                };
                let _ = reply.send(snapshot);
            }
            RoomEvent::StartGame {
                user_id,
                custom_roles,
                reply,
            } => {
                if user_id != self.host_id {
                    let _ = reply.send(StartGameResult::Error(
                        "Chỉ Host của phòng mới có thể bắt đầu trò chơi.".to_string(),
                    ));
                    return Ok(());
                }

                if self.status != RoomStatus::Waiting {
                    let _ = reply.send(StartGameResult::Error(
                        "Trò chơi đã bắt đầu hoặc kết thúc.".to_string(),
                    ));
                    return Ok(());
                }

                let roles_map = match custom_roles {
                    Some(map) => map,
                    None => {
                        let players = self.players.len() as u32;
                        let table = crate::utils::role::get_role_table(players).ok_or_else(|| {
                            anyhow::anyhow!(
                                "Không có vai trò mặc định cho {} người chơi (hỗ trợ 4-12).",
                                players
                            )
                        });

                        match table {
                            Ok(table) => {
                                let mut map: HashMap<u8, u8> = HashMap::new();
                                for (role, count) in table.iter() {
                                    if *count == 0 {
                                        continue;
                                    }
                                    let id = *role as u8;
                                    let cnt_u8: u8 = (*count)
                                        .try_into()
                                        .map_err(|_| anyhow::anyhow!("Số lượng role quá lớn"))?;
                                    map.insert(id, cnt_u8);
                                }
                                map
                            }
                            Err(e) => {
                                let _ = reply.send(StartGameResult::Error(e.to_string()));
                                return Ok(());
                            }
                        }
                    }
                };

                let total_roles: u8 = roles_map.values().sum();
                if total_roles as usize != self.players.len() {
                    let msg = format!(
                        "Tổng số vai trò ({}) phải bằng số người chơi ({}).",
                        total_roles,
                        self.players.len()
                    );
                    let _ = reply.send(StartGameResult::Error(msg));
                    return Ok(());
                }

                if *roles_map.get(&0).unwrap_or(&0) == 0 {
                    let _ = reply.send(StartGameResult::Error(
                        "Phải có ít nhất 1 Sói trong game.".to_string(),
                    ));
                    return Ok(());
                }

                if let Err(e) = self.assign_roles_and_dm(&roles_map).await {
                    let _ = reply.send(StartGameResult::Error(e.to_string()));
                    return Ok(());
                }

                let _ = reply.send(StartGameResult::Success);

                self.status = RoomStatus::Starting;
                self.game_state.phase = Phase::Night;
                self.night_phase().await?;
            }
            RoomEvent::WolfPhaseTimeout => {
                tracing::info!("Room {}: Wolf Phase Timeout", self.guild_id);
                if self.game_state.phase != Phase::Night {
                    return Ok(());
                }

                GameRoom::disable_interaction_in_phase(&self.http, &mut self.wolf_messages).await;
                self.wolf_messages.clear();
            }
            RoomEvent::EndGame => {
                self.status = RoomStatus::Ended;
            }
            RoomEvent::PhaseTimeout => {
                self.on_phase_timeout().await?;
            }
            RoomEvent::VoteComplete => {
                self.on_vote_complete().await?;
            }
            RoomEvent::WolfVote { user_id, target } => {
                self.handle_wolf_vote(user_id, target).await?;
            }
            RoomEvent::HangVote { user_id, target } => {
                self.handle_hang_vote(user_id, target).await?;
            }
            RoomEvent::WolfPhaseWarning => {
                if self.game_state.phase != Phase::Night {
                    return Ok(());
                }

                for player in self.players.iter() {
                    if player.is_werewolf() && player.alive {
                        let _ = player
                            .user_id
                            .create_dm_channel(&self.http)
                            .await?
                            .say(&self.http, "⚠️ **Còn 10 giây!** Sói hãy chốt phiếu nhanh!")
                            .await;
                    }
                }
            }
            RoomEvent::PhaseWarning => {
                let phase_promps = match self.game_state.phase {
                    Phase::Night => "trời sẽ sáng",
                    Phase::Day => "để thảo luận",
                    Phase::Voting => "sẽ chốt vote",
                    _ => "tới phase tiêp theo",
                };

                for player in self.players.iter() {
                    let _ = player
                        .user_id
                        .create_dm_channel(&self.http)
                        .await?
                        .say(
                            &self.http,
                            format!("⚠️ Còn **10 giây** nữa {}", phase_promps),
                        )
                        .await;
                }
            }
            RoomEvent::BodyguardProtect { user_id, target } => {
                if self.game_state.phase != Phase::Night {
                    return Ok(());
                }

                let (target_alive, target_name) =
                    match self.players.iter().find(|p| p.user_id == target) {
                        Some(p) => (p.alive, p.name.clone()),
                        None => return Ok(()),
                    };

                if !target_alive {
                    let _ = user_id
                        .create_dm_channel(&self.http)
                        .await?
                        .say(&self.http, "❌ Không có tác dụng lên người chết.")
                        .await;
                    return Ok(());
                }

                if let Some(player) = self.players.iter_mut().find(|p| p.user_id == user_id) {
                    if let Some(bodyguard) = player
                        .role
                        .as_any_mut()
                        .downcast_mut::<crate::roles::Bodyguard>()
                    {
                        if bodyguard.protected_count == 0 {
                            let _ = user_id
                                .create_dm_channel(&self.http)
                                .await?
                                .say(&self.http, "❌ Bạn đã hết lượt dùng chức năng.")
                                .await;
                            return Ok(());
                        }

                        bodyguard.protected_person = Some(target);

                        let _ = user_id
                            .create_dm_channel(&self.http)
                            .await?
                            .say(
                                &self.http,
                                format!("🛡️ Bạn đã bảo vệ: **{}**.", target_name),
                            )
                            .await;
                    }
                }
            }
            RoomEvent::SeerView { user_id, target } => {
                if self.game_state.phase != Phase::Night {
                    return Ok(());
                }

                let (target_alive, target_name, target_role_id, target_faction) =
                    match self.players.iter().find(|p| p.user_id == target) {
                        Some(p) => (p.alive, p.name.clone(), p.role.id(), p.role.faction()),
                        None => return Ok(()),
                    };

                if !target_alive {
                    let _ = user_id
                        .create_dm_channel(&self.http)
                        .await?
                        .say(&self.http, "❌ Không có tác dụng lên người chết.")
                        .await;
                    return Ok(());
                }

                if target == user_id {
                    let _ = user_id
                        .create_dm_channel(&self.http)
                        .await?
                        .say(&self.http, "❌ Bạn không thể xem phe của chính mình.")
                        .await;
                    return Ok(());
                }

                let alpha_masked_target = self
                    .players
                    .iter()
                    .find(|p| p.role.id() == RoleId::AlphaWerewolf && p.alive)
                    .and_then(|p| {
                        p.role
                            .as_any()
                            .downcast_ref::<crate::roles::AlphaWerewolf>()
                    })
                    .and_then(|alpha| alpha.mask_wolf);

                if let Some(player) = self.players.iter_mut().find(|p| p.user_id == user_id) {
                    if let Some(seer) = player
                        .role
                        .as_any_mut()
                        .downcast_mut::<crate::roles::Seer>()
                    {
                        if seer.view_count <= 0 {
                            let _ = user_id
                                .create_dm_channel(&self.http)
                                .await?
                                .say(&self.http, "❌ Bạn đã hết lượt dùng chức năng.")
                                .await;
                            return Ok(());
                        }

                        seer.view_count -= 1;

                        let faction_display = if Some(target) == alpha_masked_target {
                            "Dân Làng"
                        } else if target_role_id == RoleId::Lycan {
                            "Ma Sói"
                        } else {
                            match target_faction {
                                Faction::Werewolf => "Ma Sói",
                                _ => "Dân Làng",
                            }
                        };

                        let _ = user_id
                            .create_dm_channel(&self.http)
                            .await?
                            .say(
                                &self.http,
                                format!(
                                    "👁️ Phe của **{}** là: **{}**.",
                                    target_name, faction_display
                                ),
                            )
                            .await;
                    }
                }
            }
            RoomEvent::FoxSpiritFind {
                user_id,
                target1,
                target2,
                target3,
            } => {
                if self.game_state.phase != Phase::Night {
                    return Ok(());
                }

                let http = self.http.clone();
                let send_dm = move |msg: String| {
                    let http = http.clone();
                    async move {
                        if let Ok(dm) = user_id.create_dm_channel(&http).await {
                            let _ = dm.say(&http, msg).await;
                        }
                    }
                };

                let get_target_info = |uid: UserId| {
                    self.players.iter().find(|p| p.user_id == uid).map(|p| {
                        (
                            p.alive,
                            p.name.clone(),
                            p.role.faction(),
                            p.role.id(),
                            p.user_id,
                        )
                    })
                };

                let t1 = match get_target_info(target1) {
                    Some(i) => i,
                    None => return Ok(()),
                };
                let t2 = match get_target_info(target2) {
                    Some(i) => i,
                    None => return Ok(()),
                };
                let t3 = match get_target_info(target3) {
                    Some(i) => i,
                    None => return Ok(()),
                };

                if !t1.0 || !t2.0 || !t3.0 {
                    send_dm("❌ Không có tác dụng lên người chết.".to_string()).await;
                    return Ok(());
                }

                let mut fox_lost_power = false;

                if let Some(player) = self.players.iter_mut().find(|p| p.user_id == user_id) {
                    // player.has_acted = true;

                    if let Some(fox) = player
                        .role
                        .as_any_mut()
                        .downcast_mut::<crate::roles::FoxSpirit>()
                    {
                        if fox.view_count <= 0 {
                            send_dm("❌ Bạn đã hết lượt dùng chức năng.".to_string()).await;
                            return Ok(());
                        }

                        fox.view_count -= 1;
                    }
                }

                let mask_wolf_id: Option<UserId> = self
                    .players
                    .iter()
                    .find(|p| p.role.id() == RoleId::AlphaWerewolf)
                    .and_then(|p| {
                        p.role
                            .as_any()
                            .downcast_ref::<crate::roles::AlphaWerewolf>()
                    })
                    .and_then(|alpha| alpha.mask_wolf);

                let targets = vec![t1, t2, t3];
                let mut found_wolf = false;

                for (_, _, faction, role_id, uid) in &targets {
                    let is_wolf_faction = *faction == Faction::Werewolf;
                    let is_lycan = *role_id == RoleId::Lycan;
                    let is_masked = Some(*uid) == mask_wolf_id;

                    if is_lycan || (is_wolf_faction && !is_masked) {
                        found_wolf = true;
                        break;
                    }
                }

                let t_names = format!(
                    "**{}**, **{}** và **{}**",
                    targets[0].1, targets[1].1, targets[2].1
                );

                if found_wolf {
                    send_dm(format!("🦊 Trong 3 người: {}, **CÓ SÓI**.", t_names)).await;
                } else {
                    send_dm(format!("🦊 Trong 3 người: {}, **KHÔNG CÓ SÓI**.", t_names)).await;

                    send_dm("⚠️ Bạn bị mất chức năng vĩnh viễn vì đoán sai.".to_string()).await;
                    fox_lost_power = true;
                }

                if fox_lost_power {
                    if let Some(player) = self.players.iter_mut().find(|p| p.user_id == user_id) {
                        if let Some(fox) = player
                            .role
                            .as_any_mut()
                            .downcast_mut::<crate::roles::FoxSpirit>()
                        {
                            fox.view_count = 0;
                            fox.is_have_skill = false;
                        }
                    }
                }

                return Ok(());
            }
            RoomEvent::DetectiveInvestigate {
                user_id,
                target1,
                target2,
            } => {
                if self.game_state.phase != Phase::Night {
                    return Ok(());
                }

                let http = self.http.clone();
                let send_dm = |msg: String| async move {
                    if let Ok(dm) = user_id.create_dm_channel(&http).await {
                        let _ = dm.say(&http, msg).await;
                    }
                };

                let get_target_info = |uid: UserId| {
                    self.players
                        .iter()
                        .find(|p| p.user_id == uid)
                        .map(|p| (p.alive, p.name.clone(), p.role.faction(), p.role.id()))
                };

                let (t1_alive, t1_name, t1_faction, t1_role_id) = match get_target_info(target1) {
                    Some(i) => i,
                    None => return Ok(()),
                };
                let (t2_alive, t2_name, t2_faction, t2_role_id) = match get_target_info(target2) {
                    Some(i) => i,
                    None => return Ok(()),
                };
                if !t1_alive || !t2_alive {
                    send_dm("❌ Không có tác dụng lên người chết.".to_string()).await;
                    return Ok(());
                }

                if let Some(player) = self.players.iter_mut().find(|p| p.user_id == user_id) {
                    // player.has_acted = true; // Đánh dấu đã hành động (cho Stalker soi)

                    if let Some(detective) = player
                        .role
                        .as_any_mut()
                        .downcast_mut::<crate::roles::Detective>()
                    {
                        if detective.investigated_count == 0 {
                            send_dm("❌ Bạn đã hết lượt dùng chức năng.".to_string()).await;
                            return Ok(());
                        }

                        detective.investigated_count -= 1;
                        detective.investigated_targets.push(target1);
                        detective.investigated_targets.push(target2);

                        let is_same_faction = {
                            let is_lycan_involved =
                                t1_role_id == RoleId::Lycan || t2_role_id == RoleId::Lycan;

                            if is_lycan_involved {
                                let t1_is_wolf_side =
                                    t1_role_id == RoleId::Lycan || t1_faction == Faction::Werewolf;
                                let t2_is_wolf_side =
                                    t2_role_id == RoleId::Lycan || t2_faction == Faction::Werewolf;
                                t1_is_wolf_side == t2_is_wolf_side
                            } else if (t1_faction == Faction::ViWolf
                                && t2_faction == Faction::Village)
                                || (t1_faction == Faction::Village && t2_faction == Faction::ViWolf)
                            {
                                true
                            } else {
                                t1_faction == t2_faction
                            }
                        };

                        let result_text = if is_same_faction {
                            format!(
                                "🔎 Kết quả: **{}** và **{}** ở **CÙNG PHE**.",
                                t1_name, t2_name
                            )
                        } else {
                            format!(
                                "🔎 Kết quả: **{}** và **{}** ở **KHÁC PHE**.",
                                t1_name, t2_name
                            )
                        };

                        send_dm(result_text).await;
                    }
                }
                return Ok(());
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn disable_interaction_in_phase(
        http: &Arc<Http>,
        interaction_messages: &mut HashMap<UserId, Vec<(ChannelId, MessageId)>>,
    ) {
        let mut set = JoinSet::new();
        for (user_id, messages) in interaction_messages.drain() {
            for (channel_id, message_id) in messages {
                let http = http.clone();
                set.spawn(async move {
                    let msg = match channel_id.message(&http, message_id).await {
                        Ok(m) => m,
                        Err(_) => return,
                    };

                    let mut new_rows = Vec::new();
                    for row in msg.components {
                        let mut buttons = Vec::new();
                        let mut select_menu: Option<CreateSelectMenu> = None;

                        for component in row.components {
                            match component {
                                ActionRowComponent::Button(b) => {
                                    use serenity::all::ButtonKind;

                                    if let ButtonKind::NonLink { custom_id, style } = &b.data {
                                        let mut new_btn = CreateButton::new(custom_id.clone())
                                            .style(*style)
                                            .disabled(true);

                                        if let Some(label) = &b.label {
                                            new_btn = new_btn.label(label.clone());
                                        }
                                        if let Some(emoji) = &b.emoji {
                                            new_btn = new_btn.emoji(emoji.clone());
                                        }

                                        buttons.push(new_btn);
                                    }
                                }

                                ActionRowComponent::SelectMenu(m) => {
                                    let dummy_option =
                                        CreateSelectMenuOption::new("Đã hết giờ", "expired")
                                            .emoji('⌛')
                                            .default_selection(true);

                                    let new_menu = CreateSelectMenu::new(
                                        m.custom_id.clone().unwrap_or_default(),
                                        CreateSelectMenuKind::String {
                                            options: vec![dummy_option],
                                        },
                                    )
                                    .placeholder("⌛ Đã hết thời gian chọn")
                                    .disabled(true);

                                    select_menu = Some(new_menu);
                                }

                                _ => {}
                            }
                        }

                        if !buttons.is_empty() {
                            new_rows.push(CreateActionRow::Buttons(buttons));
                        } else if let Some(menu) = select_menu {
                            new_rows.push(CreateActionRow::SelectMenu(menu));
                        }
                    }
                    let edit = EditMessage::new().content(msg.content).components(new_rows);

                    if let Err(e) = channel_id.edit_message(http, message_id, edit).await {
                        tracing::warn!("Lỗi disable user {}: {:?}", user_id, e);
                    }
                });
            }
        }
        while let Some(_) = set.join_next().await {}
    }

    fn remove_player(&mut self, user_id: UserId) {
        self.players.retain(|p| p.user_id != user_id);
    }

    async fn night_phase(&mut self) -> Result<()> {
        self.game_state.phase = Phase::Night;
        self.game_state.night_count += 1;

        tracing::info!(
            "Room {}: Night phase {}",
            self.guild_id,
            self.game_state.night_count
        );

        execute_night_phase(self).await?;

        let sender_a = self.sender.clone();
        let wolf_time = self.settings.wolf_vote_time;
        tokio::spawn(async move {
            if wolf_time > 10 {
                tokio::time::sleep(std::time::Duration::from_secs(wolf_time - 10)).await;
                let _ = sender_a.send(RoomEvent::WolfPhaseWarning);

                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            } else {
                tokio::time::sleep(std::time::Duration::from_secs(wolf_time)).await;
            }
            let _ = sender_a.send(RoomEvent::WolfPhaseTimeout);
        });

        let sender_b = self.sender.clone();
        let night_time = self.settings.night_time;
        tokio::spawn(async move {
            if night_time > 10 {
                tokio::time::sleep(std::time::Duration::from_secs(night_time - 10)).await;
                let _ = sender_b.send(RoomEvent::PhaseWarning);
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            } else {
                tokio::time::sleep(std::time::Duration::from_secs(night_time)).await;
            }

            let _ = sender_b.send(RoomEvent::PhaseTimeout);
        });

        Ok(())
    }

    async fn solve_phase(&mut self) -> Result<()> {
        tracing::info!("Room {}: Solve phase", self.guild_id);
        println!("Room {}: Solve phase", self.guild_id);

        execute_solve_phase(self).await?;

        Ok(())
    }

    async fn day_phase(&mut self) -> Result<()> {
        self.game_state.phase = Phase::Day;

        tracing::info!("Room {}: Day phase", self.guild_id);

        println!("Room {}: Day phase", self.guild_id);

        execute_day_phase(self).await?;

        let sender = self.sender.clone();
        let discuss_time = self.settings.discuss_time;
        tokio::spawn(async move {
            if discuss_time > 10 {
                tokio::time::sleep(std::time::Duration::from_secs(discuss_time - 10)).await;
                let _ = sender.send(RoomEvent::PhaseWarning);
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            } else {
                tokio::time::sleep(std::time::Duration::from_secs(discuss_time)).await;
            }
            let _ = sender.send(RoomEvent::PhaseTimeout);
        });

        Ok(())
    }

    async fn vote_phase(&mut self) -> Result<()> {
        self.game_state.phase = Phase::Voting;

        tracing::info!("Room {}: Vote phase", self.guild_id);

        println!("Room {}: Vote phase", self.guild_id);

        execute_vote_phase(self).await?;

        let sender = self.sender.clone();
        let vote_time = self.settings.vote_time;
        tokio::spawn(async move {
            if vote_time > 10 {
                tokio::time::sleep(std::time::Duration::from_secs(vote_time - 10)).await;
                let _ = sender.send(RoomEvent::PhaseWarning);
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            } else {
                tokio::time::sleep(std::time::Duration::from_secs(vote_time)).await;
            }
            let _ = sender.send(RoomEvent::PhaseTimeout);
        });

        Ok(())
    }

    async fn check_end_game(&mut self) -> Result<bool> {
        if let Some(victory) = crate::game::helper::check_victory(&self.players) {
            tracing::info!(
                "Room {}: Game ended, winner: {:?}",
                self.guild_id,
                victory.winner
            );

            self.status = RoomStatus::Ended;
            return Ok(true);
        }
        Ok(false)
    }

    async fn set_phase_timer(&self, seconds: u64) {
        let sender = self.sender.clone();
        let mut cancel = self.phase_timer_cancel.lock().await;

        if let Some(handle) = cancel.take() {
            handle.abort();
        }

        let handle = tokio::spawn(async move {
            sleep(Duration::from_secs(seconds)).await;
            let _ = sender.send(RoomEvent::PhaseTimeout);
        });

        *cancel = Some(handle);
    }

    async fn on_phase_timeout(&mut self) -> Result<()> {
        tracing::info!("Room {}: Phase timeout", self.guild_id);
        if self.status != RoomStatus::Starting {
            return Ok(());
        }

        match self.game_state.phase {
            Phase::Night => {
                GameRoom::disable_interaction_in_phase(&self.http, &mut self.night_messages).await;
                self.solve_phase().await?;
                self.day_phase().await?;
            }
            Phase::Day => {
                GameRoom::disable_interaction_in_phase(&self.http, &mut self.day_messages).await;
                self.vote_phase().await?;
            }
            Phase::Voting => {
                GameRoom::disable_interaction_in_phase(&self.http, &mut self.vote_messages).await;
                self.night_phase().await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn on_vote_complete(&mut self) -> Result<()> {
        tracing::info!("Room {}: Vote complete (early)", self.guild_id);

        let mut cancel = self.phase_timer_cancel.lock().await;
        if let Some(handle) = cancel.take() {
            handle.abort();
        }

        Ok(())
    }

    async fn handle_wolf_vote(&mut self, user_id: UserId, target: UserId) -> Result<()> {
        use crate::roles::Werewolf;

        tracing::info!("Processing wolf vote: {} -> {}", user_id, target);

        let player_valid = self
            .players
            .iter()
            .find(|p| p.user_id == user_id)
            .map(|p| p.alive && p.is_werewolf())
            .unwrap_or(false);

        if !player_valid {
            let player_opt = self.players.iter().find(|p| p.user_id == user_id);
            if player_opt.is_none() {
                tracing::warn!("Wolf vote from unknown user: {}", user_id);
                anyhow::bail!("Người chơi không tồn tại");
            } else if !player_opt.unwrap().alive {
                tracing::warn!("Dead player tried to vote: {}", user_id);
                anyhow::bail!("Người chơi đã chết không thể vote");
            } else {
                tracing::warn!("Non-werewolf tried to wolf vote: {}", user_id);
                anyhow::bail!("Chỉ Ma Sói mới có thể vote cắn");
            }
        }

        let target_exists = self.players.iter().any(|p| p.user_id == target && p.alive);
        if !target_exists {
            tracing::warn!("Wolf vote target invalid: {}", target);
            anyhow::bail!("Mục tiêu không hợp lệ");
        }

        let player = self
            .players
            .iter_mut()
            .find(|p| p.user_id == user_id)
            .unwrap();

        if let Some(werewolf) = player.role.as_mut().as_any_mut().downcast_mut::<Werewolf>() {
            werewolf.vote_bite = Some(target);
            tracing::info!("Wolf {} voted to bite {}", user_id, target);
        } else {
            tracing::warn!("Failed to downcast role to Werewolf for user {}", user_id);
            anyhow::bail!("Lỗi xử lý vai trò");
        }

        Ok(())
    }

    async fn handle_hang_vote(&mut self, _user_id: UserId, _target: String) -> Result<()> {
        Ok(())
    }

    async fn cleanup(&mut self) {
        tracing::info!("Room {}: Cleaning up", self.guild_id);

        let mut cancel = self.phase_timer_cancel.lock().await;
        if let Some(handle) = cancel.take() {
            handle.abort();
        }
    }

    async fn assign_roles_and_dm(&mut self, roles_map: &HashMap<u8, u8>) -> Result<()> {
        use crate::utils::embed::create_werewolf_embed;
        use crate::utils::role::convert_faction_role;

        let mut pool: Vec<u8> = Vec::new();
        for (role_id, count) in roles_map.iter() {
            for _ in 0..*count {
                pool.push(*role_id);
            }
        }

        if pool.len() != self.players.len() {
            anyhow::bail!(
                "Tổng số vai trò ({}) phải bằng số người chơi ({}).",
                pool.len(),
                self.players.len()
            );
        }

        let mut rng = StdRng::from_entropy();
        pool.shuffle(&mut rng);

        for (player, role_id_u8) in self.players.iter_mut().zip(pool.into_iter()) {
            let role_id = crate::utils::role::RoleId::from_u8(role_id_u8)
                .ok_or_else(|| anyhow::anyhow!("Role ID không hợp lệ: {}", role_id_u8))?;
            player.role = crate::roles::create_role(role_id);
        }

        tracing::info!("Roles assigned: {:?}", self.players);

        for player in self.players.iter() {
            let role_id_u8 = player.role.id() as u8;
            let role_key = role_id_u8.to_string();

            let (title, description, file_name) = if let Some(info) = self.roles_json.get(&role_key)
            {
                let file_name = format!("{}.png", info.e_name.to_lowercase().replace(' ', "_"));
                let faction_name = convert_faction_role(info.faction);
                let desc = format!("{}\n\n**Phe:** {}", info.description, faction_name);
                (format!("{} ({})", info.title, info.e_name), desc, file_name)
            } else {
                let file_name = format!(
                    "{}.png",
                    player.role.name().to_lowercase().replace(' ', "_")
                );
                (
                    player.role.name().to_string(),
                    player.role.description().to_string(),
                    file_name,
                )
            };

            let data_embed = create_werewolf_embed(&file_name, &title, &description).await?;

            let dm = player.user_id.create_dm_channel(&self.http).await?;
            dm.send_message(
                &self.http,
                CreateMessage::new()
                    .content(format!(
                        "🎮 Bạn được phân vai: **{}**. Hãy giữ bí mật!!!",
                        player.role.name()
                    ))
                    .add_embed(data_embed.embed)
                    .add_file(data_embed.attachment),
            )
            .await?;
        }

        let wolves: Vec<&Player> = self.players.iter().filter(|p| p.is_werewolf()).collect();
        for wolf in wolves.iter() {
            let teammate_str = wolves
                .iter()
                .filter(|p| p.user_id != wolf.user_id)
                .map(|p| format!("**{}** ({})", p.name, p.role.name()))
                .collect::<Vec<_>>()
                .join(", ");

            let dm = wolf.user_id.create_dm_channel(&self.http).await?;
            dm.send_message(
                &self.http,
                CreateMessage::new().content(format!(
                    "Đồng đội của bạn: {}",
                    if teammate_str.is_empty() {
                        "Không có đồng đội.".to_string()
                    } else {
                        teammate_str
                    }
                )),
            )
            .await?;
        }

        Ok(())
    }
}

pub fn spawn_room(
    guild_id: GuildId,
    host_id: UserId,
    channel_id: ChannelId,
    ctx: &Context,
    roles_json: RolesData,
) -> RoomHandle {
    let http = ctx.http.clone();

    let (room, handle) = GameRoom::new(guild_id, host_id, channel_id, roles_json, http);

    tokio::spawn(async move {
        room.run().await;
    });

    handle
}
