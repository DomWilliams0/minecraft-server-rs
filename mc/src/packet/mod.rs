pub use handshake::*;

use crate::prelude::*;

pub type PacketId = i32;

pub struct PacketBody {
    pub id: PacketId,
    pub body: Vec<u8>,
}

pub trait Packet {
    const ID: PacketId;
}

#[async_trait]
pub trait ClientBound: Packet {
    async fn write_packet<W: McWrite>(&self, w: &mut W) -> McResult<()>;
}

#[async_trait]
pub trait ServerBound: Sized + Packet {
    async fn read_packet(body: PacketBody) -> McResult<Self>;
}

mod handshake;
mod status;
// mod login;
// mod play;

pub use status::*;
// pub use login::*;
// pub use play::*;

