use super::role::Role;
use crate::utils::role::RoleId;
use serenity::model::id::UserId;

pub struct PlayerInfo {
    pub user_id: u64,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar_url: String,
    pub alive: bool,
}

pub struct Player {
    pub name: String,
    pub user_id: UserId,
    pub alive: bool,
    pub voted: bool,
    pub role: Box<dyn Role>,
    pub can_use_skill: bool,
    pub can_vote: bool,
    pub can_chat: bool,
    pub avatar_url: String,
}

impl Player {
    pub fn new(user_id: UserId, name: String, role: Box<dyn Role>, avatar_url: String) -> Self {
        Self {
            name,
            avatar_url,
            user_id,
            alive: true,
            voted: false,
            role,
            can_use_skill: true,
            can_vote: true,
            can_chat: true,
        }
    }

    pub fn reset_round(&mut self) {
        self.voted = false;
        self.can_use_skill = true;
    }

    pub fn reset_day(&mut self) {
        self.can_use_skill = true;
    }

    pub fn reset_restrict(&mut self) {
        self.can_vote = true;
        self.can_chat = true;
    }

    pub fn is_werewolf(&self) -> bool {
        matches!(
            self.role.id(),
            RoleId::Werewolf
                | RoleId::WolfSeer
                | RoleId::AlphaWerewolf
                | RoleId::KittenWolf
                | RoleId::Voodoo
                | RoleId::Wolffluence
        )
    }
}

impl std::fmt::Debug for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Player")
            .field("name", &self.name)
            .field("user_id", &self.user_id)
            .field("alive", &self.alive)
            .field("voted", &self.voted)
            .field("role", &"<role>")
            .field("can_use_skill", &self.can_use_skill)
            .field("can_vote", &self.can_vote)
            .field("can_chat", &self.can_chat)
            .finish()
    }
}

impl Clone for Player {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            user_id: self.user_id,
            avatar_url: self.avatar_url.clone(),
            alive: self.alive,
            voted: self.voted,
            role: self.role.clone_box(),
            can_use_skill: self.can_use_skill,
            can_vote: self.can_vote,
            can_chat: self.can_chat,
        }
    }
}
