use crate::{game::room::GameRoom, utils::role::RoleId};

pub mod embed;
pub mod response;
pub mod role;
pub mod role_parser;

pub fn get_player_mut_role<T: 'static>(room: &mut GameRoom, role_id: RoleId) -> Option<&mut T> {
    room.players
        .iter_mut()
        .find(|p| p.role.id() == role_id && p.alive)?
        .role
        .as_any_mut()
        .downcast_mut::<T>()
}
