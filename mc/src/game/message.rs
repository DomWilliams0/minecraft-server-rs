use crate::packet::ClientBoundPacket;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use uuid::Uuid;

pub enum ClientMessage {
    NewClient {
        name: String,
        outgoing: UnboundedSender<ClientBoundPacket>,
    },

    VerifyTeleport(i32),
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ClientUuid(Uuid);

pub type ClientMessageSender = UnboundedSender<(ClientUuid, ClientMessage)>;
pub type ClientMessageReceiver = UnboundedReceiver<(ClientUuid, ClientMessage)>;

impl From<Uuid> for ClientUuid {
    fn from(uuid: Uuid) -> Self {
        ClientUuid(uuid)
    }
}
