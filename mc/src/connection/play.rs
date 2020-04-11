use crate::connection::{ActiveState, PlayState, ResponseSink, State};
use crate::packet::PacketBody;
use crate::prelude::*;
use crate::server::ServerData;

// TODO Keep Alive
// TODO Join Game
// TODO Chunk Data (nbt heightmaps, optional fields in packet format, some actual chunk data)
// TODO central server instance with player list,world etc, and functionality like kick()

#[async_trait]
impl<R: ResponseSink> State<R> for PlayState {
    async fn handle_transaction(
        mut self,
        packet: PacketBody,
        server_data: &ServerData,
        response_sink: &mut R,
    ) -> McResult<ActiveState> {
        match packet.id {
            x => Err(McError::BadPacketId(x)),
        }
    }
}
