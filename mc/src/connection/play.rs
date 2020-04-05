use crate::connection::{ActiveState, PlayState, State};
use crate::error::{McError, McResult};
use crate::packet::PacketBody;
use crate::server::ServerDataRef;
use std::io::Write;

pub(crate) trait PlayStateComms {
    // TODO
}

pub(crate) struct OfflinePlayState;
pub(crate) struct OnlinePlayState {
    pub shared_secret: Vec<u8>,
}

impl PlayStateComms for OfflinePlayState {}
impl PlayStateComms for OnlinePlayState {}

impl<W: Write> State<W> for PlayState {
    fn handle_transaction(
        self,
        packet: PacketBody,
        resp_write: &mut W,
        server_data: &ServerDataRef,
    ) -> McResult<ActiveState> {
        match packet.id {
            x => Err(McError::BadPacketId(x)),
        }
    }
}
