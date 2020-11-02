use crate::error::{McError, McResult};
use crate::field::StringField;
use crate::packet::{
    ClientBoundPacket, Disconnect, HeldItemChange, JoinGame, PlayerPositionAndLook, SpawnPosition,
};
use async_std::sync::Arc;
use async_std::task;
use chashmap::CHashMap;
use futures::{channel::mpsc::UnboundedSender, SinkExt, StreamExt};
use log::*;

// TODO generic sinks

mod message;

pub use message::{ClientMessage, ClientMessageReceiver, ClientMessageSender, ClientUuid};

struct Client {
    outgoing: UnboundedSender<ClientBoundPacket>,
    name: PlayerName,

    // TODO uuid to name lookup
    /// Teleport ID sent to client that should be verified with a TeleportConfirm
    next_teleport_id: Option<i32>,
}

impl Client {
    fn new(outgoing: UnboundedSender<ClientBoundPacket>, name: PlayerName) -> Self {
        Client {
            outgoing,
            name,
            next_teleport_id: None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PlayerName(String);

pub struct Game {
    clients: Arc<CHashMap<ClientUuid, Client>>,
    clients_rx: ClientMessageReceiver,
}

impl Game {
    pub fn new(clients_rx: ClientMessageReceiver) -> Self {
        Self {
            clients: Arc::new(CHashMap::with_capacity(64)),
            clients_rx,
        }
    }

    pub async fn run(mut self) -> McResult<()> {
        // client message loop
        task::spawn(async move {
            loop {
                let result;

                if let Some((uuid, msg)) = self.clients_rx.next().await {
                    match msg {
                        ClientMessage::NewClient { name, outgoing } => {
                            info!("adding player {} to the game", name);
                            self.clients
                                .insert(uuid, Client::new(outgoing, PlayerName(name)));

                            result = self.on_player_joined(uuid).await.map_err(|err| {
                                error!("failed to join player");
                                err
                            });
                        }
                        ClientMessage::VerifyTeleport(id) => {
                            result = self.check_teleport_id(uuid, id);
                        }
                    }

                    if let Err(err) = result {
                        warn!(
                            "error handling message for client, kicking {} with '{}'",
                            self.client(uuid)
                                .map(|c| c.name.0.clone()) // clone alright in exceptional case
                                .unwrap_or_else(|_| format!("player {:?}", uuid)),
                            err
                        );

                        self.kick_with_error(uuid, err).await;
                    }
                }
            }
        });

        Ok(())
    }

    fn client_mut(&self, uuid: ClientUuid) -> McResult<chashmap::WriteGuard<ClientUuid, Client>> {
        self.clients
            .get_mut(&uuid)
            .ok_or_else(|| McError::NoSuchPlayer(uuid))
    }

    fn client(&self, uuid: ClientUuid) -> McResult<chashmap::ReadGuard<ClientUuid, Client>> {
        self.clients
            .get(&uuid)
            .ok_or_else(|| McError::NoSuchPlayer(uuid))
    }

    async fn kick_with_error(&self, uuid: ClientUuid, error: McError) {
        if let Err(kick_error) = self
            .send_packet_to_client(uuid, Disconnect::with_error(&error).into())
            .await
        {
            error!(
                "failed to kick with existing error {}: {}",
                error, kick_error
            )
        }
    }

    async fn send_packet_to_client(
        &self,
        uuid: ClientUuid,
        packet: ClientBoundPacket,
    ) -> McResult<()> {
        let mut client = self.client_mut(uuid)?;
        client.outgoing.send(packet).await?;
        Ok(())
    }

    fn set_teleport_id(&self, uuid: ClientUuid, teleport_id: i32) -> McResult<()> {
        self.client_mut(uuid)?.next_teleport_id = Some(teleport_id);
        Ok(())
    }

    fn check_teleport_id(&self, uuid: ClientUuid, confirmed_teleport_id: i32) -> McResult<()> {
        let true_value = self.client_mut(uuid)?.next_teleport_id.take();
        if true_value == Some(confirmed_teleport_id) {
            Ok(())
        } else {
            Err(McError::IncorrectTeleportConfirm {
                expected: true_value,
                actual: confirmed_teleport_id,
            })
        }
    }

    #[allow(unused_variables)]
    async fn on_player_joined(&self, uuid: ClientUuid) -> McResult<()> {
        // TODO take the client lock just once
        macro_rules! send {
            ($packet:expr) => {
                self.send_packet_to_client(uuid, ClientBoundPacket::from($packet))
                    .await?;
            };
        }

        send!(JoinGame {
            entity_id: 123.into(),
            gamemode: 0.into(),
            dimension: 0.into(),
            hashed_seed: 12_345_678.into(),
            max_players: 0.into(),
            level_type: StringField::new("default".to_owned()),
            view_distance: 20.into(),
            reduced_debug_info: false.into(),
            enable_respawn_screen: true.into(),
        });

        send!(HeldItemChange { slot: 2.into() });

        send!(SpawnPosition {
            location: (500, 64, 500).into(),
        });

        let teleport_id = 1234;
        send!(PlayerPositionAndLook::new((10.0, 100.0, 10.0), teleport_id));
        self.set_teleport_id(uuid, teleport_id)?;

        Ok(())
    }
}
