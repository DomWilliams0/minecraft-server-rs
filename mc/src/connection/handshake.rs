use crate::connection::{ActiveState, HandshakeState, LoginState, StatusState};
use crate::prelude::*;
use minecraft_server_protocol::types::*;

impl HandshakeState {
    pub async fn handle_transaction(self, packet: PacketBody) -> McResult<ActiveState> {
        use crate::packet::handshaking::server::*;
        // TODO support legacy ping
        let handshake = SetProtocol::read_packet(packet).await?;

        match handshake.next_state.value() {
            1 => Ok(ActiveState::Status(StatusState::default())),
            2 => Ok(ActiveState::Login(LoginState::default())),
            x => Err(McError::BadNextState(x)),
        }
    }
}
