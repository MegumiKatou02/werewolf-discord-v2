use crate::types::{Faction, Role};
use crate::utils::role::RoleId;
use serenity::model::id::UserId;

#[derive(Debug, Clone)]
pub struct Stalker {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
    pub stalked_person: Option<UserId>,
    pub killed_person: Option<UserId>,
    pub stalk_count: u8,
    pub kill_count: u8,
}

impl Stalker {
    pub fn new() -> Self {
        Self {
            vote_hanged: None,
            death_night: -1,
            stalked_person: None,
            killed_person: None,
            stalk_count: 99,
            kill_count: 1,
        }
    }
}

impl Role for Stalker {
    fn id(&self) -> RoleId {
        RoleId::Stalker
    }
    fn faction(&self) -> Faction {
        Faction::Solo
    }
    fn description(&self) -> &'static str {
        "Mỗi đêm bạn có thể theo dõi 1 người chơi. Bạn còn có thể ám sát. Thắng khi là người duy nhất sống sót."
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
        self.stalked_person = None;
        self.killed_person = None;
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
