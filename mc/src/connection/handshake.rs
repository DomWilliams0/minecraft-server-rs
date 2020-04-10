use crate::connection::{ActiveState, HandshakeState, State, StatusState};
use crate::connection::comms::ActiveComms;
use crate::packet::*;
use crate::prelude::*;
use crate::server::ServerDataRef;

// TODO prelude

#[async_trait]
impl<S: McStream> State<S> for HandshakeState {
    async fn handle_transaction(
        self,
        packet: PacketBody,
        _server_data: &ServerDataRef,
        _comms: &mut ActiveComms<S>,
    ) -> McResult<ActiveState> {
        let handshake = Handshake::read_packet(packet).await?;

        match handshake.next_state.value() {
            1 => Ok(ActiveState::Status(StatusState::default())),
            // 2 => Ok(ActiveState::Login(LoginState::default())),
            x => Err(McError::BadNextState(x)),
        }
    }
}
