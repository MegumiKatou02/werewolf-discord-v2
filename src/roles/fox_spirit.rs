use crate::types::{Faction, Role};
use crate::utils::role::RoleId;
use serenity::model::id::UserId;

#[derive(Debug, Clone)]
pub struct FoxSpirit {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
    pub three_viewed: Vec<UserId>,
    pub is_have_skill: bool,
}

impl FoxSpirit {
    pub fn new() -> Self {
        Self {
            vote_hanged: None,
            death_night: -1,
            three_viewed: Vec::new(),
            is_have_skill: true,
        }
    }
}

impl Role for FoxSpirit {
    fn id(&self) -> RoleId {
        RoleId::FoxSpirit
    }
    fn faction(&self) -> Faction {
        Faction::Village
    }
    fn description(&self) -> &'static str {
        "Mỗi đêm dậy soi 3 người tự chọn trong danh sách, nếu 1 trong 3 người đó là sói thì được báo \"Có sói\"."
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
        self.three_viewed.clear();
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
