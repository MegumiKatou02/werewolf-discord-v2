use crate::types::{Faction, Role};
use crate::utils::role::RoleId;
use serenity::model::id::UserId;

#[derive(Debug, Clone)]
pub struct Dead {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
    pub original_role_id: RoleId,
    pub loudmouth_player: Option<UserId>,
    pub loudmouth_revealed: bool,
}

impl Dead {
    pub fn new(
        original_role_id: RoleId,
        death_night: i32,
        loudmouth_player: Option<UserId>,
    ) -> Self {
        Self {
            vote_hanged: None,
            death_night,
            original_role_id,
            loudmouth_player,
            loudmouth_revealed: false,
        }
    }

    pub fn mark_loudmouth_revealed(&mut self) {
        self.loudmouth_revealed = true;
    }
}

impl Role for Dead {
    fn id(&self) -> RoleId {
        RoleId::Dead
    }
    fn faction(&self) -> Faction {
        match self.original_role_id {
            RoleId::Werewolf
            | RoleId::WolfSeer
            | RoleId::AlphaWerewolf
            | RoleId::KittenWolf
            | RoleId::Voodoo
            | RoleId::Wolffluence => Faction::Werewolf,
            RoleId::Fool | RoleId::Stalker => Faction::Solo,
            _ => Faction::Village,
        }
    }
    fn description(&self) -> &'static str {
        "Bạn đã chết rồi, đừng hỏi gì cả..."
    }

    fn vote_hanged(&self) -> Option<String> {
        self.vote_hanged.clone()
    }
    fn set_vote_hanged(&mut self, target: Option<String>) {
        self.vote_hanged = target;
    }

    fn death_night(&self) -> i32 {
        self.death_night
    }
    fn set_death_night(&mut self, night: i32) {
        self.death_night = night;
    }

    fn reset_day(&mut self) {
        self.vote_hanged = None;
    }

    fn reset_restrict(&mut self) {}

    fn clone_box(&self) -> Box<dyn Role> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
