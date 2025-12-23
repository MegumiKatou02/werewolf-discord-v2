use crate::types::{Faction, Role};
use crate::utils::role::RoleId;
use serenity::model::id::UserId;

#[derive(Debug, Clone)]
pub struct Maid {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
    pub master: Option<UserId>,
}

impl Maid {
    pub fn new() -> Self {
        Self {
            vote_hanged: None,
            death_night: -1,
            master: None,
        }
    }
}

impl Role for Maid {
    fn id(&self) -> RoleId {
        RoleId::Maid
    }
    fn faction(&self) -> Faction {
        Faction::Village
    }
    fn description(&self) -> &'static str {
        "Bạn được chọn 1 người làm chủ, chủ chết thì người đó lên thay."
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
