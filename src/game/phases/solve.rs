use anyhow::Result;

use crate::game::room::GameRoom;

pub async fn execute_solve_phase(room: &mut GameRoom) -> Result<()> {
    tracing::info!("execute_solve_phase {:?}", room.game_state.phase);

    Ok(())
}
