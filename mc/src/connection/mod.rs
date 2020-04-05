use log::*;
use std::io::{ErrorKind, Read};

use crate::connection::comms::{ActiveComms, Stream};
use crate::error::{McError, McResult};
use crate::field::{Field, VarIntField};
use crate::packet::*;
use crate::server::ServerDataRef;
use uuid::Uuid;

mod comms;
mod handshake;
mod login;
mod play;
mod status;

trait State<S: Stream> {
    fn handle_transaction(
        self,
        packet: PacketBody,
        server_data: &ServerDataRef,
        comms: &mut ActiveComms<S>,
    ) -> McResult<ActiveState>;
}

#[derive(Default)]
struct HandshakeState;

#[derive(Default)]
struct StatusState;

#[derive(Default)]
struct LoginState {
    pub player_name: String,
    pub verify_token: Vec<u8>,
}

struct PlayState {
    pub player_name: String,
    pub uuid: Uuid,
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

pub struct ConnectionState<S: Stream> {
    comms: ActiveComms<S>,
    state: ActiveState,
    server_data: ServerDataRef,
}

impl<S: Stream> ConnectionState<S> {
    pub fn new(server_data: ServerDataRef, stream: S) -> Self {
        Self {
            comms: ActiveComms::new(stream),
            state: ActiveState::default(),
            server_data,
        }
    }
}

impl<S: Stream> ConnectionState<S> {
    pub fn handle_transaction(&mut self) -> McResult<()> {
        let packet = self.read_packet()?;

        let state = std::mem::take(&mut self.state);

        self.state = match state {
            ActiveState::Handshake(state) => {
                state.handle_transaction(packet, &self.server_data, &mut self.comms)
            }
            ActiveState::Status(state) => {
                state.handle_transaction(packet, &self.server_data, &mut self.comms)
            }
            ActiveState::Login(state) => {
                state.handle_transaction(packet, &self.server_data, &mut self.comms)
            }
            ActiveState::Play(state) => {
                state.handle_transaction(packet, &self.server_data, &mut self.comms)
            }
        }?;
        Ok(())
    }

    fn read_packet(&mut self) -> McResult<PacketBody> {
        let mut length = match VarIntField::read(&mut self.comms) {
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
            let varint = VarIntField::read(&mut self.comms)?;
            length -= varint.size() as i32; // length includes packet id
            varint.value()
        };

        debug!("packet id={:#x}", packet_id);

        let mut recv_buf = vec![0u8; length as usize]; // TODO somehow reuse a buffer in self without making borrowck shit itself
        if length > 0 {
            self.comms.read_exact(&mut recv_buf).map_err(McError::Io)?;
        }

        Ok(PacketBody {
            id: packet_id,
            body: recv_buf,
        })
    }
}
