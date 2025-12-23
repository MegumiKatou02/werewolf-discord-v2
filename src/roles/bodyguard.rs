use crate::types::{Faction, Role};
use crate::utils::role::RoleId;
use serenity::model::id::UserId;

#[derive(Debug, Clone)]
pub struct Bodyguard {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
    pub protected_person: Option<UserId>,
    pub protected_count: u8,
    pub hp: u8,
}

impl Bodyguard {
    pub fn new() -> Self {
        Self {
            vote_hanged: None,
            death_night: -1,
            protected_person: None,
            protected_count: 1,
            hp: 2,
        }
    }
}

impl Role for Bodyguard {
    fn id(&self) -> RoleId {
        RoleId::Bodyguard
    }
    fn faction(&self) -> Faction {
        Faction::Village
    }
    fn description(&self) -> &'static str {
        "Bạn có thể chọn một người chơi để bảo vệ mỗi đêm."
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
        self.protected_person = None;
        self.protected_count = 1;
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
