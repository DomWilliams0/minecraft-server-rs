use crate::error::McResult;
use std::io::Write;

pub type PacketId = i32;

pub struct PacketBody<'a> {
    pub id: PacketId,
    pub body: &'a [u8],
}

pub trait Packet {
    fn id() -> PacketId;
}

pub trait ClientBound: Packet {
    fn write<W: Write>(&self, w: &mut W) -> McResult<()>;
}

pub trait ServerBound: Sized + Packet {
    fn read(body: PacketBody) -> McResult<Self>;
}

pub mod handshake;
pub mod status;
