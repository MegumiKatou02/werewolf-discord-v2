use crate::{
    impl_basic_role,
    types::{Faction, Role},
    utils::role::RoleId,
};

#[derive(Debug, Clone)]
pub struct Cursed {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
}

impl Cursed {
    pub fn new() -> Self {
        Self {
            vote_hanged: None,
            death_night: -1,
        }
    }
}

impl_basic_role!(
    Cursed,
    RoleId::Cursed,
    Faction::ViWolf,
    "Bạn là dân làng bình thường cho tới khi bị ma sói cắn, lúc đó bạn sẽ trở thành Ma sói."
);
