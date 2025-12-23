use crate::types::{Faction, Role};
use crate::utils::role::RoleId;

#[derive(Debug, Clone)]
pub struct Seer {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
    pub view_count: u8,
}

impl Seer {
    pub fn new() -> Self {
        Self {
            vote_hanged: None,
            death_night: -1,
            view_count: 1,
        }
    }
}

impl Role for Seer {
    fn id(&self) -> RoleId {
        RoleId::Seer
    }
    fn faction(&self) -> Faction {
        Faction::Village
    }
    fn description(&self) -> &'static str {
        "Mỗi đêm bạn có thể xem phe của người chơi khác."
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
        self.view_count = 1;
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
