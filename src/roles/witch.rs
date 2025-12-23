use crate::types::{Faction, Role};
use crate::utils::role::RoleId;
use serenity::model::id::UserId;

#[derive(Debug, Clone)]
pub struct Witch {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
    pub poison_count: u8,
    pub heal_count: u8,
    pub poisoned_person: Option<UserId>,
    pub healed_person: Option<UserId>,
    pub need_help_person: Option<UserId>,
}

impl Witch {
    pub fn new() -> Self {
        Self {
            vote_hanged: None,
            death_night: -1,
            poison_count: 1,
            heal_count: 1,
            poisoned_person: None,
            healed_person: None,
            need_help_person: None,
        }
    }
}

impl Role for Witch {
    fn id(&self) -> RoleId {
        RoleId::Witch
    }
    fn faction(&self) -> Faction {
        Faction::Village
    }
    fn description(&self) -> &'static str {
        "Bạn có hai bình thuốc: Một bình dùng để giết và bình kia để bảo vệ người chơi."
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
        self.poisoned_person = None;
        self.healed_person = None;
        self.need_help_person = None;
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
