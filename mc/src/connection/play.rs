use crate::connection::{ActiveState, PlayState};
use crate::field::*;
use crate::game::{ClientMessage, ClientMessageSender};
use crate::packet::*;
use crate::prelude::*;
use async_std::io::Cursor;

use futures::SinkExt;

// TODO Keep Alive
// TODO Join Game
// TODO Chunk Data (nbt heightmaps, optional fields in packet format, some actual chunk data)
// TODO central server instance with player list,world etc, and functionality like kick()

impl PlayState {
    pub async fn handle_transaction(
        self,
        packet: PacketBody,
        game_broker: &mut ClientMessageSender,
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

            TeleportConfirm::ID => {
                let confirmation = TeleportConfirm::read_packet(packet).await?;
                game_broker
                    .send((
                        self.uuid,
                        ClientMessage::VerifyTeleport(confirmation.teleport_id.value()),
                    ))
                    .await?;
                Ok(())
            }

            PlayerPositionAndRotation::ID => {
                let _pos = PlayerPositionAndRotation::read_packet(packet).await?;
                // TODO actually use
                Ok(())
            }

            KeepAliveResponse::ID => {
                let keep_alive = KeepAliveResponse::read_packet(packet).await?;
                game_broker
                    .send((
                        self.uuid,
                        ClientMessage::VerifyKeepAlive(*keep_alive.keep_alive_id.value()),
                    ))
                    .await?;

                Ok(())
            }

            x => Err(McError::BadPacketId(x)),
        }?;

        Ok(ActiveState::Play(self))
    }
}
