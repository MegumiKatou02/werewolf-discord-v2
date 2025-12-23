use crate::types::{Faction, Role};
use crate::utils::role::RoleId;
use serenity::model::id::UserId;

#[derive(Debug, Clone)]
pub struct VoodooWerewolf {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
    pub vote_bite: Option<UserId>,
    pub silent_player: Option<UserId>,
    pub voodoo_player: Option<UserId>,
    pub silent_count: u8,
    pub voodoo_count: u8,
}

impl VoodooWerewolf {
    pub fn new() -> Self {
        Self {
            vote_hanged: None,
            death_night: -1,
            vote_bite: None,
            silent_player: None,
            voodoo_player: None,
            silent_count: 2,
            voodoo_count: 1,
        }
    }
}

impl Role for VoodooWerewolf {
    fn id(&self) -> RoleId {
        RoleId::Voodoo
    }
    fn faction(&self) -> Faction {
        Faction::Werewolf
    }
    fn description(&self) -> &'static str {
        "Bạn có thể làm câm lặng và đưa người chơi vào ác mộng."
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
        self.vote_bite = None;
        self.silent_player = None;
        self.voodoo_player = None;
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
