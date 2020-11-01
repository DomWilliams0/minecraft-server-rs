use crate::connection::comms::{CommsRef, ResponseSink};
use crate::connection::{ActiveState, PlayState, State};
use crate::field::*;
use crate::packet::*;
use crate::prelude::*;
use crate::server::ServerData;
use async_std::io::Cursor;

// TODO Keep Alive
// TODO Join Game
// TODO Chunk Data (nbt heightmaps, optional fields in packet format, some actual chunk data)
// TODO central server instance with player list,world etc, and functionality like kick()

#[async_trait]
impl<R: ResponseSink> State<R> for PlayState {
    async fn handle_transaction(
        mut self,
        packet: PacketBody,
        _server_data: &ServerData,
        _comms: &mut CommsRef<R>,
    ) -> McResult<ActiveState> {
        match packet.id {
            ClientSettings::ID => {
                let _client_settings = ClientSettings::read_packet(packet).await?;
                // whatever
                Ok(())
            }

            PluginMessage::ID => {
                let plugin_message = PluginMessage::read_packet(packet).await?;

                let value = match (
                    plugin_message.channel.namespace(),
                    plugin_message.channel.location(),
                ) {
                    ("minecraft", "brand") => {
                        let mut cursor = Cursor::new(&plugin_message.data.value().0);
                        let string = StringField::read_field(&mut cursor).await?;
                        string.take()
                    }
                    _ => "unknown".to_owned(),
                };

                debug!(
                    "got plugin message: namespace={}, location={}, value={}",
                    plugin_message.channel.namespace(),
                    plugin_message.channel.location(),
                    value
                );
                Ok(())
            }
            x => Err(McError::BadPacketId(x)),
        }?;

        Ok(ActiveState::Play(self))
    }
}
