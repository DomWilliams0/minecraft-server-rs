use crate::error::McResult;
use std::io::Write;

pub type PacketId = i32;

pub struct PacketBody {
    pub id: PacketId,
    pub body: Vec<u8>,
}

pub trait Packet {
    const ID: PacketId;
}

pub trait ClientBound: Packet {
    fn write<W: Write>(&self, w: &mut W) -> McResult<()>;
}

pub trait ServerBound: Sized + Packet {
    fn read(body: PacketBody) -> McResult<Self>;
}

mod handshake;
mod login;
mod status;

pub use handshake::*;
pub use login::*;
pub use status::*;
