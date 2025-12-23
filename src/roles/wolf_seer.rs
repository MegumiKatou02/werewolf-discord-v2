use crate::{
    impl_basic_role,
    types::{Faction, Role},
    utils::role::RoleId,
};

#[derive(Debug, Clone)]
pub struct WolfSeer {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
}

impl WolfSeer {
    pub fn new() -> Self {
        Self {
            vote_hanged: None,
            death_night: -1,
        }
    }
}

impl_basic_role!(
    WolfSeer,
    RoleId::WolfSeer,
    Faction::Werewolf,
    "Soi xem ai là tiên tri."
);
