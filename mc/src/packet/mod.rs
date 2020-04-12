use std::ops::Deref;

use async_std::io::Cursor;

use crate::prelude::*;

pub type PacketId = i32;

pub struct PacketBody {
    pub id: PacketId,
    pub body: Vec<u8>,
}

pub trait Packet: Send + Sync {
    // fn id() -> PacketId;
}

#[async_trait]
pub trait ClientBound: Packet {
    async fn write_packet(&self, w: &mut Cursor<&mut [u8]>) -> McResult<()>;

    fn length(&self) -> usize;

    fn full_size(&self) -> usize {
        let len = VarIntField::new(self.length() as i32);
        len.value() as usize + len.size()
    }
}

#[async_trait]
pub trait ServerBound: Sized + Packet {
    // TODO make this sync and block on reading
    async fn read_packet(body: PacketBody) -> McResult<Self>;
}

// TODO arena allocator
pub struct ClientBoundPacket(Box<dyn ClientBound>);

impl<P: ClientBound + 'static> From<P> for ClientBoundPacket {
    fn from(packet: P) -> Self {
        Self(Box::new(packet))
    }
}

impl Deref for ClientBoundPacket {
    type Target = dyn ClientBound;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

mod handshake;
mod login;
mod play;
mod status;

use crate::field::{Field, VarIntField};
pub use handshake::*;
pub use login::*;
pub use play::*;
pub use status::*;
