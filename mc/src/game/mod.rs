use std::hint::unreachable_unchecked;
use std::time::Duration;

use async_std::sync::Arc;
use async_std::task;
use async_std::task::JoinHandle;
use chashmap::{CHashMap, WriteGuard};
use futures::{channel::mpsc::UnboundedSender, SinkExt, StreamExt};
use log::*;
use packets::types::*;

pub use message::{ClientMessage, ClientMessageReceiver, ClientMessageSender, ClientUuid};

use crate::error::{McError, McResult};
use crate::packet::play::client as play;
use crate::packet::play::client::KickDisconnect;
use crate::packet::PlayerPositionAndLookExt;
use crate::packet::{DisconnectExt, KeepAliveExt};

// TODO generic sinks

mod message;

struct Client {
    outgoing: UnboundedSender<ClientBoundPacket>,
    name: PlayerName,

    // TODO uuid to name lookup
    /// Teleport ID sent to client that should be verified with a TeleportConfirm
    next_teleport_id: Option<i32>,

    keep_alive: JoinHandle<()>,
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
                if let Some((uuid, mut msg)) = self.clients_rx.next().await {
                    // special case where client doesn't exist already
                    if let ClientMessage::NewClient { .. } = &msg {
                        // take ownership of this msg and replace with body-less variant

                        let new_client = std::mem::replace(&mut msg, ClientMessage::PlayerJoined);
                        match new_client {
                            ClientMessage::NewClient { name, outgoing } => {
                                self.add_player(uuid, name, outgoing).await;
                            }
                            _ => unsafe {
                                // just checked
                                unreachable_unchecked()
                            },
                        };
                    }
                    // special case where client is removed with no further processing
                    else if let ClientMessage::PlayerDisconnected = &msg {
                        self.remove_player(uuid).await;

                        // nothing else to do
                        continue;
                    }

                    let result = match self.client_mut(uuid) {
                        Err(err) => Err(err),
                        Ok(client) => self.handle_message(client, msg).await,
                    };

                    if let Err(err) = result {
                        error!("error handling message for client {:?}: {}", uuid, err);

                        match self.client_mut(uuid) {
                            Err(err) => warn!("can't kick client: {}", err),
                            Ok(mut client) => {
                                info!("kicking player {} with error message", client.name.0);
                                client.kick_with_error(err).await;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    async fn add_player(
        &mut self,
        uuid: ClientUuid,
        name: String,
        outgoing: UnboundedSender<ClientBoundPacket>,
    ) {
        // spawn keep-alive task
        let mut keep_alive_tx = outgoing.clone();
        let keep_alive = task::spawn(async move {
            loop {
                task::sleep(Duration::from_secs(1)).await;

                if let Err(err) = keep_alive_tx.send(play::KeepAlive::generate().into()).await {
                    warn!("failed to send keep-alive: {}", err);
                    break;
                }
            }
        });

        // add to clients map
        let count = self.clients.len();
        info!(
            "adding player {} to the game, now has {} players",
            name,
            count + 1
        );

        let client = Client {
            outgoing,
            name: PlayerName(name),
            keep_alive,
            next_teleport_id: None,
        };
        self.clients.insert(uuid, client);
    }

    async fn remove_player(&mut self, uuid: ClientUuid) {
        match self.clients.remove(&uuid) {
            Some(client) => {
                let count = self.clients.len();
                info!(
                    "removed player {} from the game, now has {} players",
                    client.name.0, count
                );

                // stop keep-alive task
                let _ = client.keep_alive.cancel().await;
            }
            None => warn!("player {:?} disconnected but was not joined", uuid),
        };
    }

    async fn handle_message(
        &self,
        mut client: WriteGuard<'_, ClientUuid, Client>,
        msg: ClientMessage,
    ) -> McResult<()> {
        use ClientMessage::*;

        match msg {
            NewClient { .. } | PlayerDisconnected => unreachable!(),

            PlayerJoined => client.on_player_joined().await.map_err(|err| {
                error!("failed to join player");
                err
            }),
            VerifyTeleport(id) => client.check_teleport_id(id),
            VerifyKeepAlive(keep_alive) => {
                // TODO check keep alive value properly
                if keep_alive == 4 {
                    Ok(())
                } else {
                    Err(McError::IncorrectKeepAlive(keep_alive))
                }
            }
        }
    }

    fn client_mut(&self, uuid: ClientUuid) -> McResult<chashmap::WriteGuard<ClientUuid, Client>> {
        self.clients
            .get_mut(&uuid)
            .ok_or(McError::NoSuchPlayer(uuid))
    }

    // fn client(&self, uuid: ClientUuid) -> McResult<chashmap::ReadGuard<ClientUuid, Client>> {
    //     self.clients
    //         .get(&uuid)
    //         .ok_or_else(|| McError::NoSuchPlayer(uuid))
    // }
}

impl Client {
    async fn kick_with_error(&mut self, error: McError) {
        if let Err(kick_error) = self
            .send_packet(KickDisconnect::with_error(&error).into())
            .await
        {
            error!(
                "failed to kick with existing error {}: {}",
                error, kick_error
            )
        }
    }

    async fn send_packet(&mut self, packet: ClientBoundPacket) -> McResult<()> {
        self.outgoing.send(packet).await?;
        Ok(())
    }

    fn set_teleport_id(&mut self, teleport_id: i32) {
        self.next_teleport_id = Some(teleport_id);
    }

    fn check_teleport_id(&mut self, confirmed_teleport_id: i32) -> McResult<()> {
        let true_value = self.next_teleport_id.take();
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
    async fn on_player_joined(&mut self) -> McResult<()> {
        macro_rules! send {
            ($packet:expr) => {
                self.send_packet(ClientBoundPacket::from($packet)).await?;
            };
        }

        send!(play::Login {
            entity_id: 123.into(),
            game_mode: 0.into(),
            dimension: 0.into(),
            hashed_seed: 12_345_678.into(),
            max_players: 0.into(),
            level_type: StringField::new("default".to_owned()),
            view_distance: 20.into(),
            reduced_debug_info: false.into(),
            enable_respawn_screen: true.into(),
        });

        // TODO handle held item slot response
        // send!(play::HeldItemSlot { slot: 2.into() });

        send!(play::SpawnPosition {
            location: PositionField::new((500, 64, -500)).unwrap(),
        });

        let teleport_id = 1234;
        send!(play::Position::new((10.0, 100.0, -10.0), teleport_id));
        self.set_teleport_id(teleport_id);

        Ok(())
    }
}
