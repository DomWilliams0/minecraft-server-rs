use std::io::Write;

use crate::connection::{ActiveState, HandshakeState, LoginState, State, StatusState};
use crate::error::{McError, McResult};
use crate::packet::PacketBody;
use crate::packet::*;

impl<W: Write> State<W> for HandshakeState {
    fn handle_transaction(self, packet: PacketBody, _resp_write: &mut W) -> McResult<ActiveState> {
        let handshake = Handshake::read(packet)?;

        match handshake.next_state.value() {
            1 => Ok(ActiveState::Status(StatusState)),
            2 => Ok(ActiveState::Login(LoginState)),
            x => Err(McError::BadNextState(x)),
        }
    }
}
