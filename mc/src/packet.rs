use crate::error::{McError, McResult};
use crate::field::{Field, StringField, UShortField, VarIntField};
use mc_packet_derive::ServerBoundPacket;
use std::io::Cursor;

pub type PacketId = i32;

pub struct PacketBody<'a> {
    pub id: PacketId,
    pub body: &'a [u8],
}

pub trait Packet {
    fn id() -> PacketId;
}

pub trait ClientBound: Sized + Packet {}

pub trait ServerBound: Sized + Packet {
    fn read(body: PacketBody) -> McResult<Self>;
}

#[derive(ServerBoundPacket, Debug)]
#[packet_id = 0x00]
pub struct PacketHandshake {
    pub protocol_version: VarIntField,
    pub server_address: StringField,
    pub server_port: UShortField,
    pub next_state: VarIntField,
}

// impl Packet for PacketHandshake {
//     fn id() -> PacketId {
//         0x00
//     }
// }
//
// impl ServerBound for PacketHandshake {
//     fn read(body: PacketBody) -> McResult<Self> {
//         if body.id != Self::id() {
//             return Err(McError::UnexpectedPacket {
//                 expected: Self::id(),
//                 actual: body.id,
//             });
//         }
//         let mut cursor = Cursor::new(body.body);
//
//         let protocol_version = VarIntField::read(&mut cursor)?;
//         let server_address = StringField::read(&mut cursor)?;
//         let server_port = UShortField::read(&mut cursor)?;
//         let next_state = VarIntField::read(&mut cursor)?;
//
//         Ok(Self {
//             protocol_version,
//             server_address,
//             server_port,
//             next_state,
//         })
//     }
// }
