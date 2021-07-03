use crate::connection::comms::{CommsRef, ResponseSink};
use crate::connection::{ActiveState, StatusState};
use crate::packet::*;
use crate::prelude::*;
use packets::types::*;

impl StatusState {
    pub async fn handle_transaction<R: ResponseSink>(
        self,
        packet: PacketBody,
        comms: &mut CommsRef<R>,
    ) -> McResult<ActiveState> {
        use crate::packet::status::{client, server::*};
        match packet.id {
            PingStart::ID => {
                let _empty = PingStart::read_packet(packet).await?;

                let status = client::ServerInfo {
                    response: StringField::new(generate_json(
                        "mInEcRaFt",
                        include_str!("../../icon.png.base64"),
                    )),
                };

                comms.send_response(status).await?;

                Ok(ActiveState::Status(self))
            }
            Ping::ID => {
                let ping = Ping::read_packet(packet).await?;
                let pong = client::Ping { time: ping.time };
                comms.send_response(pong).await?;

                Err(McError::PleaseDisconnect)
            }
            x => Err(McError::BadPacketId(x)),
        }
    }
}

fn generate_json(description: &str, icon_b64: &str) -> String {
    format!(
        "{{\"version\": {{\"name\": \"{game_version}\", \"protocol\": {protocol_version} }},\
    \"players\": {{ \"max\": 10, \"online\": 7, \"sample\": [] }}, \"description\": {{ \"text\": \"{desc}\" }},\
    \"favicon\": \"data:image/png;base64,{icon}\"}}",
        game_version = GAME_VERSION,
        protocol_version = PROTOCOL_VERSION,
        desc = description,
        icon = icon_b64
    )
}
