use crate::connection::{ActiveState, HandshakeState, LoginState, StatusState};
use crate::packet::*;
use crate::prelude::*;

impl HandshakeState {
    pub async fn handle_transaction(self, packet: PacketBody) -> McResult<ActiveState> {
        let handshake = Handshake::read_packet(packet).await?;

        match handshake.next_state.value() {
            1 => Ok(ActiveState::Status(StatusState::default())),
            2 => Ok(ActiveState::Login(LoginState::default())),
            x => Err(McError::BadNextState(x)),
        }
    }
}
