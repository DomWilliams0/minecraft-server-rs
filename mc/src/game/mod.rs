use crate::error::{McError, McResult};
use crate::field::StringField;
use crate::packet::{ClientBoundPacket, JoinGame};
use async_std::sync::Arc;
use async_std::task;
use chashmap::CHashMap;
use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    SinkExt, StreamExt,
};
use log::*;
use uuid::Uuid;

// TODO generic sinks

struct Client {
    // incoming: UnboundedReceiver<>
    outgoing: UnboundedSender<ClientBoundPacket>,
    uuid: Uuid,
}

impl Client {}

pub type PlayerName = String;

pub struct Game {
    clients: Arc<CHashMap<PlayerName, Client>>,
    clients_rx: UnboundedReceiver<ClientMessage>,
}

pub enum ClientMessage {
    NewClient {
        name: PlayerName,
        uuid: Uuid,
        outgoing: UnboundedSender<ClientBoundPacket>,
    },
}

impl Game {
    pub fn new(clients_rx: UnboundedReceiver<ClientMessage>) -> Self {
        Self {
            clients: Arc::new(CHashMap::with_capacity(64)),
            clients_rx,
        }
    }

    pub async fn run(mut self) -> McResult<()> {
        // client message loop
        task::spawn(async move {
            loop {
                if let Some(msg) = self.clients_rx.next().await {
                    match msg {
                        ClientMessage::NewClient {
                            name,
                            uuid,
                            outgoing,
                        } => {
                            info!("adding player {} to the game", name);
                            self.clients.insert(name.clone(), Client { outgoing, uuid });

                            if let Err(e) = self.on_player_joined(&name).await {
                                error!("error adding player to the game: {}", e);
                                // TODO kick?
                            }
                        }
                    }
                }
            }
        });
        //
        // loop{
        //     task::sleep(Duration::from_secs(1_000_000_000)).await;
        // }

        Ok(())
    }

    async fn send_packet_to_client(
        &self,
        name: &PlayerName,
        packet: ClientBoundPacket,
    ) -> McResult<()> {
        let mut client = self
            .clients
            .get_mut(name)
            .ok_or_else(|| McError::NoSuchPlayer(name.clone()))?;
        client
            .outgoing
            .send(packet)
            .await
            .map_err(|_| McError::Sink)
    }

    async fn on_player_joined(&self, player: &PlayerName) -> McResult<()> {
        let join_game = JoinGame {
            entity_id: 123.into(),
            gamemode: 0.into(),
            dimension: 0.into(),
            hashed_seed: 12_345_678.into(),
            max_players: 0.into(),
            level_type: StringField::new("default".to_owned()),
            view_distance: 20.into(),
            reduced_debug_info: false.into(),
            enable_respawn_screen: true.into(),
        };

        self.send_packet_to_client(player, join_game.into()).await?;

        Ok(())
    }
}
