use super::faction::Faction;
use crate::utils::role::RoleId;

pub trait Role: Send + Sync {
    fn id(&self) -> RoleId;
    fn name(&self) -> &'static str {
        self.id().name()
    }
    fn faction(&self) -> Faction;
    fn description(&self) -> &'static str;

    fn vote_hanged(&self) -> Option<String>;
    fn set_vote_hanged(&mut self, target: Option<String>);

    fn death_night(&self) -> i32;
    fn set_death_night(&mut self, night: i32);

    fn reset_day(&mut self);
    fn reset_restrict(&mut self);

    fn clone_box(&self) -> Box<dyn Role>;

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl Clone for Box<dyn Role> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
