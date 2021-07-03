use async_std::io::Cursor;
use futures::SinkExt;
use packets::types::*;

use crate::connection::{ActiveState, PlayState};
use crate::game::{ClientMessage, ClientMessageSender};
use crate::prelude::*;
use packets::types::IdentifierField;

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
        use crate::packet::play::{client, server::*};
        match packet.id {
            Settings::ID => {
                let _client_settings = Settings::read_packet(packet).await?;
                // whatever
                Ok(())
            }

            CustomPayload::ID => {
                let plugin_message = CustomPayload::read_packet(packet).await?;
                let channel = IdentifierField::from(plugin_message.channel);

                let value = match (channel.namespace(), channel.location()) {
                    ("minecraft", "brand") => {
                        let mut cursor = Cursor::new(&plugin_message.data.value().0);
                        let string = StringField::read_field(&mut cursor).await?;
                        string.take()
                    }
                    _ => "unknown".to_owned(),
                };

                debug!(
                    "got plugin message: namespace={}, location={}, value={}",
                    channel.namespace(),
                    channel.location(),
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

            PositionLook::ID => {
                let _pos = PositionLook::read_packet(packet).await?;
                // TODO actually use
                Ok(())
            }

            KeepAlive::ID => {
                let keep_alive = KeepAlive::read_packet(packet).await?;
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
