use crate::types::{Faction, Role};
use crate::utils::role::RoleId;
use serenity::model::id::UserId;

#[derive(Debug, Clone)]
pub struct Medium {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
    pub revived_person: Option<UserId>,
    pub revived_count: u8,
}

impl Medium {
    pub fn new() -> Self {
        Self {
            vote_hanged: None,
            death_night: -1,
            revived_person: None,
            revived_count: 1,
        }
    }
}

impl Role for Medium {
    fn id(&self) -> RoleId {
        RoleId::Medium
    }
    fn faction(&self) -> Faction {
        Faction::Village
    }
    fn description(&self) -> &'static str {
        "Vào buổi đêm bạn có thể trò chuyện ẩn danh với người chết. Bạn có khả năng chọn một dân làng đã chết trong đêm và hồi sinh họ."
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
        self.revived_person = None;
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
