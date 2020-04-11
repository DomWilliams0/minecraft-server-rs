use crate::connection::comms::{CommsRef, ResponseSink};
use crate::connection::{ActiveState, State, StatusState};
use crate::field::*;
use crate::packet::*;
use crate::prelude::*;
use crate::server::ServerData;

#[async_trait]
impl<R: ResponseSink> State<R> for StatusState {
    async fn handle_transaction(
        self,
        packet: PacketBody,
        _server_data: &ServerData,
        comms: &mut CommsRef<R>,
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

                comms.send_response(status).await?;

                Ok(ActiveState::Status(self))
            }
            Ping::ID => {
                let ping = Ping::read_packet(packet).await?;
                let pong = Pong {
                    payload: ping.payload,
                };
                comms.send_response(pong).await?;

                Err(McError::PleaseDisconnect)
            }
            x => Err(McError::BadPacketId(x)),
        }
    }
}

fn generate_json(description: &str, icon_b64: &str) -> String {
    format!("{{\"version\": {{\"name\": \"1.15.2\", \"protocol\": 578 }}, \"players\": {{ \"max\": 10, \"online\": 7 }}, \"description\": {{ \"text\": \"{}\" }}, \"favicon\": \"data:image/png;base64,{}\"}}", description, icon_b64)
}
