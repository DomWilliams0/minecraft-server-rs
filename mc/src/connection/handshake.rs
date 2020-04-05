use crate::connection::comms::{ActiveComms, Stream};
use crate::connection::{ActiveState, HandshakeState, LoginState, State, StatusState};
use crate::error::{McError, McResult};
use crate::packet::PacketBody;
use crate::packet::*;
use crate::server::ServerDataRef;

impl<S: Stream> State<S> for HandshakeState {
    fn handle_transaction(
        self,
        packet: PacketBody,
        _server_data: &ServerDataRef,
        _comms: &mut ActiveComms<S>,
    ) -> McResult<ActiveState> {
        let handshake = Handshake::read(packet)?;

        match handshake.next_state.value() {
            1 => Ok(ActiveState::Status(StatusState::default())),
            2 => Ok(ActiveState::Login(LoginState::default())),
            x => Err(McError::BadNextState(x)),
        }
    }
}
