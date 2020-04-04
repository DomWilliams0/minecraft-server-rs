use log::*;
use std::io::{ErrorKind, Read, Write};

use crate::error::{McError, McResult};
use crate::field::{Field, VarIntField};
use crate::packet::*;
use crate::server::ServerDataRef;

mod handshake;
mod login;
mod status;

trait State<W: Write> {
    fn handle_transaction(self, packet: PacketBody, resp_write: &mut W) -> McResult<ActiveState>;
}

#[derive(Copy, Clone)]
struct HandshakeState;

#[derive(Copy, Clone)]
struct StatusState;

#[derive(Copy, Clone)]
struct LoginState;

enum ActiveState {
    Handshake(HandshakeState),
    Status(StatusState),
    Login(LoginState),
}

pub struct ConnectionState<S: Read + Write> {
    // TODO server_config: ServerConfigRef,
    stream: S,
    state: ActiveState,
    server_data: ServerDataRef,
}

impl<S: Read + Write> ConnectionState<S> {
    pub fn new(server_data: ServerDataRef, stream: S) -> Self {
        Self {
            stream,
            state: ActiveState::Handshake(HandshakeState),
            server_data,
        }
    }
}

impl<S: Read + Write> ConnectionState<S> {
    pub fn handle_transaction(&mut self) -> McResult<()> {
        let packet = self.read_packet()?;

        self.state = match &self.state {
            ActiveState::Handshake(state) => state.handle_transaction(packet, &mut self.stream),
            ActiveState::Status(state) => state.handle_transaction(packet, &mut self.stream),
            ActiveState::Login(state) => state.handle_transaction(packet, &mut self.stream),
        }?;
        Ok(())
    }

    fn read_packet(&mut self) -> McResult<PacketBody> {
        let mut length = match VarIntField::read(&mut self.stream) {
            Err(McError::Io(e)) if e.kind() == ErrorKind::UnexpectedEof => {
                debug!("eof");
                return Err(McError::PleaseDisconnect);
            }

            Err(e) => return Err(e),
            Ok(len) => len.value(),
        };

        if length < 1 || length > 65535 {
            return Err(McError::BadPacketLength(length as usize));
        }

        debug!("packet length={}", length);

        let packet_id = {
            let varint = VarIntField::read(&mut self.stream)?;
            length -= varint.size() as i32; // length includes packet id
            varint.value()
        };

        debug!("packet id={:#x}", packet_id);

        let mut recv_buf = vec![0u8; length as usize]; // TODO somehow reuse a buffer in self without making borrowck shit itself
        if length > 0 {
            self.stream.read_exact(&mut recv_buf).map_err(McError::Io)?;
        }

        Ok(PacketBody {
            id: packet_id,
            body: recv_buf,
        })
    }
}
