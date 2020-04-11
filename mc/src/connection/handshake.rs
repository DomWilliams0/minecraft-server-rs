use crate::connection::comms::{CommsRef, ResponseSink};
use crate::connection::{ActiveState, HandshakeState, LoginState, State, StatusState};
use crate::packet::*;
use crate::prelude::*;
use crate::server::ServerData;

#[async_trait]
impl<R: ResponseSink> State<R> for HandshakeState {
    async fn handle_transaction(
        self,
        packet: PacketBody,
        _server_data: &ServerData,
        _comms: &mut CommsRef<R>,
    ) -> McResult<ActiveState> {
        let handshake = Handshake::read_packet(packet).await?;

        match handshake.next_state.value() {
            1 => Ok(ActiveState::Status(StatusState::default())),
            2 => Ok(ActiveState::Login(LoginState::default())),
            x => Err(McError::BadNextState(x)),
        }
    }
}
