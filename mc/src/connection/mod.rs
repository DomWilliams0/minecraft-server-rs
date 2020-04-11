use std::ops::DerefMut;

use async_std::io::ErrorKind;
use async_std::sync::Mutex;
use futures::{Sink, SinkExt};
use uuid::Uuid;

use crate::connection::comms::ActiveComms;
use crate::field::*;
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
pub trait ResponseSink: Sink<ClientBoundPacket> + Unpin + Send {
    async fn send_packet<P: ClientBound + Sync + Send>(&mut self, packet: P) -> McResult<()> {
        let c = ClientBoundPacket::from(packet);
        self.send(c).await.map_err(|_| McError::ResponseSink)
    }
}

impl<S: Sink<ClientBoundPacket> + Unpin + Send> ResponseSink for S {}

#[async_trait]
trait State<R: ResponseSink> {
    async fn handle_transaction(
        self,
        packet: PacketBody,
        server_data: &ServerData,
        response_sink: &mut R,
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

pub struct ConnectionState<S: McStream, R: ResponseSink> {
    comms: Mutex<ActiveComms<S>>,
    state: ActiveState,
    response_sink: R,
}

impl<S: McStream, R: ResponseSink> ConnectionState<S, R> {
    pub fn new(stream: S, response_sink: R) -> Self {
        Self {
            comms: Mutex::new(ActiveComms::new(stream)),
            state: ActiveState::default(),
            response_sink,
        }
    }

    pub async fn read_packet(&self) -> McResult<PacketBody> {
        let mut comms = self.comms.lock().await;
        let comms = comms.deref_mut();
        let mut length = match VarIntField::read_field(comms).await {
            Err(McError::Io(e)) if e.kind() == ErrorKind::UnexpectedEof => {
                debug!("eof");
                return Err(McError::PleaseDisconnect);
            }

            Err(e) => return Err(e),
            Ok(len) => len.value(),
        };

        if length < 1 || length > 65535 {
            return Err(McError::BadPacketLength(length as usize));
        }

        debug!("packet length={}", length);

        let packet_id = {
            let varint = VarIntField::read_field(comms).await?;
            length -= varint.size() as i32; // length includes packet id
            varint.value()
        };

        debug!("packet id={:#x}", packet_id);

        let mut recv_buf = vec![0u8; length as usize]; // TODO somehow reuse a buffer in self without making borrowck shit itself
        if length > 0 {
            comms.read_exact(&mut recv_buf).await.map_err(McError::Io)?;
        }

        Ok(PacketBody {
            id: packet_id,
            body: recv_buf,
        })
    }

    pub async fn handle_packet(
        &mut self,
        packet: PacketBody,
        server_data: &ServerData,
    ) -> McResult<()> {
        let state = std::mem::take(&mut self.state); // TODO is this safe?

        self.state = match state {
            ActiveState::Handshake(state) => {
                state
                    .handle_transaction(packet, server_data, &mut self.response_sink)
                    .await
            }
            ActiveState::Status(state) => {
                state
                    .handle_transaction(packet, server_data, &mut self.response_sink)
                    .await
            }
            ActiveState::Login(state) => {
                state
                    .handle_transaction(packet, server_data, &mut self.response_sink)
                    .await
            }
            ActiveState::Play(state) => {
                state
                    .handle_transaction(packet, server_data, &mut self.response_sink)
                    .await
            }
        }?;
        Ok(())
    }
}
