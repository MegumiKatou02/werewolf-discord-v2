use crate::types::{Faction, Role};
use crate::utils::role::RoleId;
use serenity::model::id::UserId;

#[derive(Debug, Clone)]
pub struct AlphaWerewolf {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
    pub mask_wolf: Option<UserId>,
}

impl AlphaWerewolf {
    pub fn new() -> Self {
        Self {
            vote_hanged: None,
            death_night: -1,
            mask_wolf: None,
        }
    }
}

impl Role for AlphaWerewolf {
    fn id(&self) -> RoleId {
        RoleId::AlphaWerewolf
    }
    fn faction(&self) -> Faction {
        Faction::Werewolf
    }
    fn description(&self) -> &'static str {
        "Che các sói khỏi tiên tri, mỗi đêm 1 sói."
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
        self.mask_wolf = None;
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
