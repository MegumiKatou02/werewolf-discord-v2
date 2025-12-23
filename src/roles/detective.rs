use serenity::all::UserId;

use crate::{
    types::{Faction, Role},
    utils::role::RoleId,
};

#[derive(Debug, Clone)]
pub struct Detective {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
    pub investigated_count: u8,
    pub investigated_targets: Vec<UserId>,
}

impl Detective {
    pub fn new() -> Self {
        Self {
            vote_hanged: None,
            death_night: -1,
            investigated_count: 1,
            investigated_targets: Vec::new(),
        }
    }
}

impl Role for Detective {
    fn id(&self) -> RoleId {
        RoleId::Detective
    }
    fn faction(&self) -> Faction {
        Faction::Village
    }
    fn description(&self) -> &'static str {
        "Mỗi đêm, bạn có thể chọn hai người chơi để điều tra và biết được họ ở cùng một phe hay là khác phe."
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
        self.investigated_count = 1;
        self.investigated_targets.clear();
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
