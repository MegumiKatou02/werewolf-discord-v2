use crate::{
    impl_basic_role,
    types::{Faction, Role},
    utils::role::RoleId,
};

#[derive(Debug, Clone)]
pub struct Lycan {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
}

impl Lycan {
    pub fn new() -> Self {
        Self {
            vote_hanged: None,
            death_night: -1,
        }
    }
}

impl_basic_role!(
    Lycan,
    RoleId::Lycan,
    Faction::Village,
    "Là dân, bị soi thì quản trò báo tiên tri là phe sói."
);
