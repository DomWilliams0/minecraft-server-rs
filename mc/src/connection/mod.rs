use log::*;
use std::io::{ErrorKind, Read, Write};

use crate::connection::play::PlayStateComms;
use crate::error::{McError, McResult};
use crate::field::{Field, VarIntField};
use crate::packet::*;
use crate::server::ServerDataRef;
use uuid::Uuid;

mod handshake;
mod login;
mod play;
mod status;

trait State<W: Write> {
    fn handle_transaction(
        self,
        packet: PacketBody,
        resp_write: &mut W,
        server_data: &ServerDataRef,
    ) -> McResult<ActiveState>;
}

#[derive(Copy, Clone, Default)]
struct HandshakeState;

#[derive(Copy, Clone, Default)]
struct StatusState;

#[derive(Default)]
struct LoginState {
    pub player_name: String,
    pub verify_token: Vec<u8>,
}

struct PlayState {
    pub player_name: String,
    pub uuid: Uuid,
    pub comms: Box<dyn PlayStateComms>,
}

enum ActiveState {
    Handshake(HandshakeState),
    Status(StatusState),
    Login(LoginState),
    Play(PlayState),
}

impl Default for ActiveState {
    fn default() -> Self {
        ActiveState::Handshake(HandshakeState::default())
    }
}

pub struct ConnectionState<S: Read + Write> {
    stream: S,
    state: ActiveState,
    server_data: ServerDataRef,
}

impl<S: Read + Write> ConnectionState<S> {
    pub fn new(server_data: ServerDataRef, stream: S) -> Self {
        Self {
            stream,
            state: ActiveState::default(),
            server_data,
        }
    }
}

impl<S: Read + Write> ConnectionState<S> {
    pub fn handle_transaction(&mut self) -> McResult<()> {
        let packet = self.read_packet()?;

        let state = std::mem::take(&mut self.state);

        self.state = match state {
            ActiveState::Handshake(state) => {
                state.handle_transaction(packet, &mut self.stream, &self.server_data)
            }
            ActiveState::Status(state) => {
                state.handle_transaction(packet, &mut self.stream, &self.server_data)
            }
            ActiveState::Login(state) => {
                state.handle_transaction(packet, &mut self.stream, &self.server_data)
            }
            ActiveState::Play(state) => {
                state.handle_transaction(packet, &mut self.stream, &self.server_data)
            }
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
