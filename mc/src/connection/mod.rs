use uuid::Uuid;

pub use comms::{ActiveComms, CommsRef};

use crate::connection::comms::ResponseSink;
use crate::packet::*;
use crate::prelude::*;
use crate::server::ServerData;

mod comms;
mod handshake;
mod login;
mod play;
mod status;

pub trait McRead: Read + Unpin + Send {}
pub trait McWrite: Write + Unpin + Send {}
pub trait McStream: McRead + McWrite {}

impl<T: Read + Unpin + Send> McRead for T {}
impl<T: Write + Unpin + Send> McWrite for T {}
impl<T: McRead + McWrite> McStream for T {}

#[async_trait]
trait State<R: ResponseSink> {
    async fn handle_transaction(
        self,
        packet: PacketBody,
        server_data: &ServerData,
        response_sink: &mut CommsRef<R>,
    ) -> McResult<ActiveState>;
}

#[derive(Default)]
struct HandshakeState;

#[derive(Default)]
struct StatusState;

#[derive(Default)]
struct LoginState {
    pub player_name: String,
    pub verify_token: Vec<u8>,
}

struct PlayState {
    pub player_name: String,
    pub uuid: Uuid,
}

enum ActiveState {
    Handshake(HandshakeState),
    Status(StatusState),
    Login(LoginState),
    Play(PlayState),
}

impl Default for ActiveState {
    fn default() -> Self {
        ActiveState::Handshake(HandshakeState::default())
    }
}

pub struct ConnectionState<R: ResponseSink> {
    state: ActiveState,
    comms: CommsRef<R>, // TODO rename
}

pub enum PostPacketAction {
    None,
    EnteredPlayState {
        player_name: String,
        player_uuid: Uuid,
    },
}

impl Default for PostPacketAction {
    fn default() -> Self {
        PostPacketAction::None
    }
}

impl<R: ResponseSink> ConnectionState<R> {
    pub fn new(comms: CommsRef<R>) -> Self {
        Self {
            state: ActiveState::default(),
            comms,
        }
    }

    pub async fn handle_packet(
        &mut self,
        packet: PacketBody,
        server_data: &ServerData,
    ) -> McResult<PostPacketAction> {
        let state = std::mem::take(&mut self.state); // TODO is this safe?

        let mut action = PostPacketAction::default();
        self.state = match state {
            ActiveState::Handshake(state) => {
                state
                    .handle_transaction(packet, server_data, &mut self.comms)
                    .await
            }
            ActiveState::Status(state) => {
                state
                    .handle_transaction(packet, server_data, &mut self.comms)
                    .await
            }
            ActiveState::Login(state) => {
                let result = state
                    .handle_transaction(packet, server_data, &mut self.comms)
                    .await;

                if let Ok(ActiveState::Play(play)) = &result {
                    action = PostPacketAction::EnteredPlayState {
                        player_name: play.player_name.clone(),
                        player_uuid: play.uuid,
                    };
                }

                result
            }
            ActiveState::Play(state) => {
                state
                    .handle_transaction(packet, server_data, &mut self.comms)
                    .await
            }
        }?;
        Ok(action)
    }
}
