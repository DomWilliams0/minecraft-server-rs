use crate::error::{McError, McResult};

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
    pub fn handle(self, packet: &[u8]) -> McResult<Connection> {
        match self {
            Connection::Handshake(state) => state.handle(packet),
            Connection::Status(state) => state.handle(packet),
        }
    }
}

impl Handshake {
    pub fn handle(self, packet: &[u8]) -> McResult<Connection> {
        let next_state = 1;

        match next_state {
            1 => Ok(Connection::Status(Status)),
            2 => unimplemented!(),
            _ => Err(McError::BadNextState(next_state)),
        }
    }
}

impl Status {
    pub fn handle(self, packet: &[u8]) -> McResult<Connection> {
        todo!()
    }
}
