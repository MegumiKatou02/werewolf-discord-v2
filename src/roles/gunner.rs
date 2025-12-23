use crate::types::{Faction, Role};
use crate::utils::role::RoleId;

#[derive(Debug, Clone)]
pub struct Gunner {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
    pub bullets: u8,
}

impl Gunner {
    pub fn new() -> Self {
        Self {
            vote_hanged: None,
            death_night: -1,
            bullets: 2,
        }
    }
}

impl Role for Gunner {
    fn id(&self) -> RoleId {
        RoleId::Gunner
    }
    fn faction(&self) -> Faction {
        Faction::Village
    }
    fn description(&self) -> &'static str {
        "Bạn có hai viên đạn mà bạn có thể sử dụng để bắn ai đó."
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
