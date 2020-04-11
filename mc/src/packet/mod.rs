use std::ops::Deref;

use async_std::io::Cursor;

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

    fn size(&self) -> usize;
}

#[async_trait]
pub trait ServerBound: Sized + Packet {
    // TODO make this sync and block on reading
    async fn read_packet(body: PacketBody) -> McResult<Self>;
}

// TODO arena allocator
pub struct ClientBoundPacket(Box<[u8]>);

impl<P: ClientBound> From<P> for ClientBoundPacket {
    fn from(packet: P) -> Self {
        let mut cursor = Cursor::new(vec![]);
        async_std::task::block_on(packet.write_packet(&mut cursor))
            .expect("writing packet to cursor should not fail"); // TODO really?
        Self(cursor.into_inner().into_boxed_slice())
    }
}

impl From<Vec<u8>> for ClientBoundPacket {
    fn from(vec: Vec<u8>) -> Self {
        Self(vec.into_boxed_slice())
    }
}

impl Deref for ClientBoundPacket {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

mod handshake;
mod login;
mod play;
mod status;

pub use handshake::*;
pub use login::*;
pub use play::*;
pub use status::*;
