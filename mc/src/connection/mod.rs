use std::ops::DerefMut;

use async_std::io::ErrorKind;
use async_std::sync::Mutex;
use uuid::Uuid;

use crate::connection::comms::ActiveComms;
use crate::field::*;
use crate::packet::*;
use crate::prelude::*;
use crate::server::ServerDataRef;

mod comms;
mod handshake;
mod status;
// mod login;
// mod play;

pub trait McRead: Read + Unpin + Send {}
pub trait McWrite: Write + Unpin + Send {}
pub trait McStream: McRead + McWrite {}

impl<T: Read + Unpin + Send> McRead for T {}
impl<T: Write + Unpin + Send> McWrite for T {}
impl<T: McRead + McWrite> McStream for T {}

#[async_trait]
trait State<S: McStream> {
    async fn handle_transaction(
        self,
        packet: PacketBody,
        server_data: &ServerDataRef,
        comms: &mut ActiveComms<S>,
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
    // Login(LoginState),
    // Play(PlayState),
}

impl Default for ActiveState {
    fn default() -> Self {
        ActiveState::Handshake(HandshakeState::default())
    }
}

pub struct ConnectionState<S: McStream> {
    comms: Mutex<ActiveComms<S>>,
    state: ActiveState,
    server_data: ServerDataRef,
}

impl<S: McStream> ConnectionState<S> {
    pub fn new(server_data: ServerDataRef, stream: S) -> Self {
        Self {
            comms: Mutex::new(ActiveComms::new(stream)),
            state: ActiveState::default(),
            server_data,
        }
    }
}

impl<S: McStream> ConnectionState<S> {
    pub async fn handle_transaction(&mut self) -> McResult<()> {
        let packet = self.read_packet().await?;

        let state = std::mem::take(&mut self.state);

        let mut comms = self.comms.lock().await;
        self.state = match state {
            ActiveState::Handshake(state) => {
                state
                    .handle_transaction(packet, &self.server_data, &mut comms)
                    .await
            }
            ActiveState::Status(state) => {
                state
                    .handle_transaction(packet, &self.server_data, &mut comms)
                    .await
            }
            // ActiveState::Login(state) => {
            //     state
            //         .handle_transaction(packet, &self.server_data, &mut comms)
            //         .await
            // },
        }?;
        Ok(())
    }

    async fn read_packet(&mut self) -> McResult<PacketBody> {
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
}
