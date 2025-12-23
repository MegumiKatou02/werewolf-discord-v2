pub mod canvas;
pub mod faction;
pub mod helper;
pub mod phases;
pub mod room;
pub mod state;

use std::{collections::HashMap, sync::Arc};
use tokio::sync::oneshot;

pub use room::RoomRegistry;
use serenity::all::{ChannelId, MessageId, UserId};
pub use state::GameState;

use crate::types::Player;

#[derive(Debug, Clone)]
pub struct ChatFile {
    pub url: String,
    pub filename: String,
    pub size: u32,
}

struct RawFile {
    data: Arc<Vec<u8>>,
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoomStatus {
    Waiting,
    Starting,
    Ended,
}

#[derive(Debug, Clone)]
pub struct RoomSnapshot {
    pub status: RoomStatus,
    pub host_id: UserId,
    pub players: Vec<Player>,
    pub game_state: GameState,
}

#[derive(Debug)]
pub enum JoinResult {
    Success(usize),
    RoomFull,
    AlreadyJoined,
    GameStarted,
    WrongChannel(ChannelId, UserId),
}

#[derive(Debug)]
pub enum LeaveResult {
    Success(usize),
    RoomEmpty,
    NotJoined,
    GameStarted,
}

#[derive(Debug)]
pub enum StartGameResult {
    Success,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageTypeStore {
    WolfMessage,
    NightMessage,
    DayMessage,
    VoteMessage,
}

#[derive(Debug)]
pub enum RoomEvent {
    RegisterInteraction {
        user_id: UserId,
        channel_id: ChannelId,
        message_id: MessageId,
        message_type_store: MessageTypeStore,
    },
    WolfPhaseWarning,
    PhaseWarning,
    JoinRequest {
        user_id: UserId,
        name: String,
        avatar_url: String,
        channel_id: ChannelId,
        reply: oneshot::Sender<JoinResult>,
    },
    LeaveRequest {
        user_id: UserId,
        reply: oneshot::Sender<LeaveResult>,
    },
    StatusRequest {
        reply: oneshot::Sender<RoomSnapshot>,
    },

    StartGame {
        user_id: UserId,
        custom_roles: Option<HashMap<u8, u8>>,
        reply: oneshot::Sender<StartGameResult>,
    },
    EndGame,

    PhaseTimeout,
    VoteComplete,

    WolfVote {
        user_id: UserId,
        target: UserId,
    },
    WolfChat {
        sender_id: UserId,
        sender_name: String,
        content: String,
    },
    DayChat {
        sender_id: UserId,
        sender_name: String,
        content: String,
        attachments: Vec<ChatFile>,
    },
    WolfPhaseTimeout,
    SeerView {
        user_id: UserId,
        target: UserId,
    },
    WitchPoison {
        user_id: UserId,
        target: UserId,
    },
    WitchHeal {
        user_id: UserId,
        target: UserId,
    },
    BodyguardProtect {
        user_id: UserId,
        target: UserId,
    },
    HangVote {
        user_id: UserId,
        target: String,
    },
    GetAllPlayers {
        reply: tokio::sync::oneshot::Sender<Vec<Player>>,
    },
}

#[derive(Clone)]
pub struct RoomSettings {
    pub wolf_vote_time: u64,
    pub night_time: u64,
    pub discuss_time: u64,
    pub vote_time: u64,
}

impl Default for RoomSettings {
    fn default() -> Self {
        Self {
            wolf_vote_time: 40,
            night_time: 70,
            discuss_time: 90,
            vote_time: 30,
        }
    }
}
