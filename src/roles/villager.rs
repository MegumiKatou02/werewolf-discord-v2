use crate::{
    impl_basic_role,
    types::{Faction, Role},
    utils::role::RoleId,
};

#[derive(Debug, Clone)]
pub struct Villager {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
}

impl Villager {
    pub fn new() -> Self {
        Self {
            vote_hanged: None,
            death_night: -1,
        }
    }
}

impl_basic_role!(
    Villager,
    RoleId::Villager,
    Faction::Village,
    "Bạn là một dân làng bình thường và không có khả năng gì đặc biệt."
);
