use crate::connection::{ActiveState, PlayState, State};
use crate::connection::comms::ActiveComms;
use crate::error::{McError, McResult};
use crate::packet::PacketBody;
use crate::server::ServerDataRef;

// TODO Keep Alive
// TODO Join Game
// TODO Chunk Data (nbt heightmaps, optional fields in packet format, some actual chunk data)
// TODO central server instance with player list,world etc, and functionality like kick()

impl<S: McStream> State<S> for PlayState {
    fn handle_transaction(
        self,
        packet: PacketBody,
        _server_data: &ServerDataRef,
        _comms: &mut ActiveComms<S>,
    ) -> McResult<ActiveState> {
        match packet.id {
            x => Err(McError::BadPacketId(x)),
        }
    }
}
