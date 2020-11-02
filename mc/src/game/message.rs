use crate::packet::ClientBoundPacket;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use uuid::Uuid;

pub enum ClientMessage {
    NewClient {
        name: String,
        outgoing: UnboundedSender<ClientBoundPacket>,
    },

    /// Body-less variant of NewClient
    PlayerJoined,

    /// TeleportConfirm
    VerifyTeleport(i32),

    /// Notification that player has disconnected
    PlayerDisconnected,
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
