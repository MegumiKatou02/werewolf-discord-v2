use crate::{
    impl_basic_role,
    types::{Faction, Role},
    utils::role::RoleId,
};

#[derive(Debug, Clone)]
pub struct Detective {
    pub vote_hanged: Option<String>,
    pub death_night: i32,
}

impl Detective {
    pub fn new() -> Self {
        Self {
            vote_hanged: None,
            death_night: -1,
        }
    }
}

impl_basic_role!(
    Detective,
    RoleId::Detective,
    Faction::Village,
    "Mỗi đêm, bạn có thể chọn hai người chơi để điều tra và biết được họ ở cùng một phe hay là khác phe."
);
