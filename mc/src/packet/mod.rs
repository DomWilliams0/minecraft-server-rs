use async_std::io::Write;

use async_trait::async_trait;
pub use handshake::*;

use crate::error::McResult;

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
    async fn write_packet<W: Write + Unpin + Send>(&self, w: &mut W) -> McResult<()>;
}

#[async_trait]
pub trait ServerBound: Sized + Packet {
    async fn read_packet(body: PacketBody) -> McResult<Self>;
}

mod handshake;
// mod login;
// mod status;
// mod play;

// pub use login::*;
// pub use status::*;
// pub use play::*;
