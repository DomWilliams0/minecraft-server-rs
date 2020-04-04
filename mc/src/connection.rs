use crate::error::{McError, McResult};
use crate::field::StringField;
use crate::packet::handshake::PacketHandshake;
use crate::packet::status::{PacketEmpty, PacketPing, PacketPong, PacketStatusResponse};
use crate::packet::{ClientBound, PacketBody, ServerBound};
use std::io::Write;

pub struct Handshake;

pub struct Status;

pub enum Connection {
    Handshake(Handshake),
    Status(Status),
    Disconnect,
}

impl Default for Connection {
    fn default() -> Self {
        Connection::Handshake(Handshake)
    }
}

impl Connection {
    pub fn handle<W: Write>(self, packet: PacketBody, resp_write: &mut W) -> McResult<Connection> {
        match self {
            Connection::Handshake(state) => state.handle(packet, resp_write),
            Connection::Status(state) => state.handle(packet, resp_write),
            _ => Err(McError::Disconnected),
        }
    }
}

impl Handshake {
    pub fn handle<W: Write>(self, packet: PacketBody, _resp_write: &mut W) -> McResult<Connection> {
        let handshake = PacketHandshake::read(packet)?;

        match handshake.next_state.value() {
            1 => Ok(Connection::Status(Status)),
            2 => unimplemented!(), // TODO play
            x => Err(McError::BadNextState(x)),
        }
    }
}

impl Status {
    pub fn handle<W: Write>(self, packet: PacketBody, resp_write: &mut W) -> McResult<Connection> {
        match packet.id {
            0 => {
                let _empty = PacketEmpty::read(packet)?;

                let response = PacketStatusResponse {
                    json_response: StringField::new(generate_json(
                        "mInEcRaFt",
                        include_str!("../icon.png.base64"),
                    )),
                };

                response.write(resp_write)?;
                Ok(Connection::Status(self))
            }
            1 => {
                let ping = PacketPing::read(packet)?;
                let response = PacketPong {
                    payload: ping.payload,
                };

                response.write(resp_write)?;
                Ok(Connection::Disconnect)
            }
            x => Err(McError::BadPacketId(x)),
        }
    }
}

fn generate_json(description: &str, icon_b64: &str) -> String {
    format!("{{\"version\": {{\"name\": \"1.15.2\", \"protocol\": 578 }}, \"players\": {{ \"max\": 10, \"online\": 7 }}, \"description\": {{ \"text\": \"{}\" }}, \"favicon\": \"data:image/png;base64,{}\"}}", description, icon_b64)
}
