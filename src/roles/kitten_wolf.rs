use crate::{
    impl_basic_role,
    types::{Faction, Role},
    utils::role::RoleId,
};

#[derive(Debug, Clone)]
pub struct KittenWolf {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
}

impl KittenWolf {
    pub fn new() -> Self {
        Self {
            vote_hanged: None,
            death_night: -1,
        }
    }
}

impl_basic_role!(
    KittenWolf,
    RoleId::KittenWolf,
    Faction::Werewolf,
    "Bạn là một ma sói. Khi bạn bị giết, vote sói tiếp theo sẽ biến đổi dân làng thành ma sói."
);
