use async_std::io::prelude::*;
use async_trait::async_trait;

use crate::connection::{ActiveState, HandshakeState, LoginState, State, StatusState};
use crate::connection::comms::ActiveComms;
use crate::error::{McError, McResult};
use crate::field::StringField;
use crate::packet::*;
use crate::packet::PacketBody;
use crate::server::ServerDataRef;

#[async_trait]
impl<S: Read + Write + Unpin + Send> State<S> for StatusState {
    async fn handle_transaction(
        self,
        packet: PacketBody,
        _server_data: &ServerDataRef,
        comms: &mut ActiveComms<S>,
    ) -> McResult<ActiveState> {
        match packet.id {
            Empty::ID => {
                let _empty = Empty::read_packet(packet).await?;

                let status = StatusResponse {
                    json_response: StringField::new(generate_json(
                        "mInEcRaFt",
                        include_str!("../../icon.png.base64"),
                    )),
                };

                status.write_packet(comms).await?;
                Ok(ActiveState::Status(self))
            }
            Ping::ID => {
                let ping = Ping::read_packet(packet).await?;
                let pong = Pong {
                    payload: ping.payload,
                };

                pong.write_packet(comms).await?;
                Err(McError::PleaseDisconnect)
            }
            x => Err(McError::BadPacketId(x)),
        }
    }
}

fn generate_json(description: &str, icon_b64: &str) -> String {
    format!("{{\"version\": {{\"name\": \"1.15.2\", \"protocol\": 578 }}, \"players\": {{ \"max\": 10, \"online\": 7 }}, \"description\": {{ \"text\": \"{}\" }}, \"favicon\": \"data:image/png;base64,{}\"}}", description, icon_b64)
}
