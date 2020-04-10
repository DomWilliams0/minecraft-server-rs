use crate::connection::{ActiveState, ResponseSink, State, StatusState};
use crate::field::*;
use crate::packet::*;
use crate::prelude::*;
use crate::server::ServerDataRef;

#[async_trait]
impl<R: ResponseSink> State<R> for StatusState {
    async fn handle_transaction(
        self,
        packet: PacketBody,
        _server_data: &ServerDataRef,
        response_sink: &mut R,
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

                response_sink.send_packet(status).await?;

                Ok(ActiveState::Status(self))
            }
            Ping::ID => {
                let ping = Ping::read_packet(packet).await?;
                let pong = Pong {
                    payload: ping.payload,
                };
                response_sink.send_packet(pong).await?;

                Err(McError::PleaseDisconnect)
            }
            x => Err(McError::BadPacketId(x)),
        }
    }
}

fn generate_json(description: &str, icon_b64: &str) -> String {
    format!("{{\"version\": {{\"name\": \"1.15.2\", \"protocol\": 578 }}, \"players\": {{ \"max\": 10, \"online\": 7 }}, \"description\": {{ \"text\": \"{}\" }}, \"favicon\": \"data:image/png;base64,{}\"}}", description, icon_b64)
}
