use crate::{
    impl_basic_role,
    types::{Faction, Role},
    utils::role::RoleId,
};

#[derive(Debug, Clone)]
pub struct Fool {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
}

impl Fool {
    pub fn new() -> Self {
        Self {
            vote_hanged: None,
            death_night: -1,
        }
    }
}

impl_basic_role!(
    Fool,
    RoleId::Fool,
    Faction::Solo,
    "Bạn phải lừa dân làng treo cổ bạn. Nếu họ treo cổ bạn, bạn thắng."
);
