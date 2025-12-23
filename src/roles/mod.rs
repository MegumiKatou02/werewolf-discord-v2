mod alpha_werewolf;
mod bodyguard;
mod cursed;
mod dead;
mod detective;
mod elder;
mod fool;
mod fox_spirit;
mod gunner;
mod kitten_wolf;
mod loudmouth;
mod lycan;
mod maid;
mod medium;
mod puppeteer;
mod seer;
mod stalker;
mod villager;
mod voodoo;
mod werewolf;
mod witch;
mod wolf_seer;
mod wolffluence;

pub use alpha_werewolf::AlphaWerewolf;
pub use bodyguard::Bodyguard;
pub use cursed::Cursed;
pub use dead::Dead;
pub use detective::Detective;
pub use elder::Elder;
pub use fool::Fool;
pub use fox_spirit::FoxSpirit;
pub use gunner::Gunner;
pub use kitten_wolf::KittenWolf;
pub use loudmouth::Loudmouth;
pub use lycan::Lycan;
pub use maid::Maid;
pub use medium::Medium;
pub use puppeteer::Puppeteer;
pub use seer::Seer;
pub use stalker::Stalker;
pub use villager::Villager;
pub use voodoo::VoodooWerewolf;
pub use werewolf::Werewolf;
pub use witch::Witch;
pub use wolf_seer::WolfSeer;
pub use wolffluence::Wolffluence;

use crate::types::Role;
use crate::utils::role::RoleId;

/// Factory function để tạo role từ RoleId
pub fn create_role(role_id: RoleId) -> Box<dyn Role> {
    match role_id {
        RoleId::Werewolf => Box::new(Werewolf::new()),
        RoleId::Villager => Box::new(Villager::new()),
        RoleId::Bodyguard => Box::new(Bodyguard::new()),
        RoleId::Cursed => Box::new(Cursed::new()),
        RoleId::Seer => Box::new(Seer::new()),
        RoleId::Detective => Box::new(Detective::new()),
        RoleId::Witch => Box::new(Witch::new()),
        RoleId::Fool => Box::new(Fool::new()),
        RoleId::Medium => Box::new(Medium::new()),
        RoleId::Dead => Box::new(Dead::new(RoleId::Villager, -1, None)),
        RoleId::Maid => Box::new(Maid::new()),
        RoleId::Lycan => Box::new(Lycan::new()),
        RoleId::WolfSeer => Box::new(WolfSeer::new()),
        RoleId::AlphaWerewolf => Box::new(AlphaWerewolf::new()),
        RoleId::FoxSpirit => Box::new(FoxSpirit::new()),
        RoleId::Elder => Box::new(Elder::new()),
        RoleId::Stalker => Box::new(Stalker::new()),
        RoleId::Gunner => Box::new(Gunner::new()),
        RoleId::KittenWolf => Box::new(KittenWolf::new()),
        RoleId::Puppeteer => Box::new(Puppeteer::new()),
        RoleId::Voodoo => Box::new(VoodooWerewolf::new()),
        RoleId::Wolffluence => Box::new(Wolffluence::new()),
        RoleId::Loudmouth => Box::new(Loudmouth::new()),
    }
}

/// Macro để giảm boilerplate cho basic roles
#[macro_export]
macro_rules! impl_basic_role {
    ($role:ident, $id:expr, $faction:expr, $desc:literal) => {
        impl $crate::types::Role for $role {
            fn id(&self) -> $crate::utils::role::RoleId {
                $id
            }
            fn faction(&self) -> $crate::types::Faction {
                $faction
            }
            fn description(&self) -> &'static str {
                $desc
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
            }

            fn reset_restrict(&mut self) {}

            fn clone_box(&self) -> Box<dyn $crate::types::Role> {
                Box::new(self.clone())
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };
}

pub fn assign_roles() {}
