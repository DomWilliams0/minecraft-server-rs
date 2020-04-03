use crate::error::{McError, McResult};
use crate::packet::{PacketBody, PacketHandshake, ServerBound};

pub struct Handshake;

pub struct Status;

pub enum Connection {
    Handshake(Handshake),
    Status(Status),
}

impl Default for Connection {
    fn default() -> Self {
        Connection::Handshake(Handshake)
    }
}

impl Connection {
    pub fn handle(self, packet: PacketBody) -> McResult<Connection> {
        match self {
            Connection::Handshake(state) => state.handle(packet),
            Connection::Status(state) => state.handle(packet),
        }
    }
}

impl Handshake {
    pub fn handle(self, packet: PacketBody) -> McResult<Connection> {
        let handshake = PacketHandshake::read(packet)?;

        use log::*;
        warn!("nice {:?}", handshake);
        match handshake.next_state.value() {
            1 => Ok(Connection::Status(Status)),
            2 => unimplemented!(),
            x => Err(McError::BadNextState(x)),
        }
    }
}

impl Status {
    pub fn handle(self, packet: PacketBody) -> McResult<Connection> {
        todo!()
    }
}
