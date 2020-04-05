use crate::connection::comms::{ActiveComms, Stream};
use crate::connection::{ActiveState, State, StatusState};
use crate::error::{McError, McResult};
use crate::field::*;
use crate::packet::*;
use crate::server::ServerDataRef;

impl<S: Stream> State<S> for StatusState {
    fn handle_transaction(
        self,
        packet: PacketBody,
        _server_data: &ServerDataRef,
        comms: &mut ActiveComms<S>,
    ) -> McResult<ActiveState> {
        match packet.id {
            Empty::ID => {
                let _empty = Empty::read(packet)?;

                let status = StatusResponse {
                    json_response: StringField::new(generate_json(
                        "mInEcRaFt",
                        include_str!("../../icon.png.base64"),
                    )),
                };

                status.write(comms)?;
                Ok(ActiveState::Status(self))
            }
            Ping::ID => {
                let ping = Ping::read(packet)?;
                let pong = Pong {
                    payload: ping.payload,
                };

                pong.write(comms)?;
                Err(McError::PleaseDisconnect)
            }
            x => Err(McError::BadPacketId(x)),
        }
    }
}

fn generate_json(description: &str, icon_b64: &str) -> String {
    format!("{{\"version\": {{\"name\": \"1.15.2\", \"protocol\": 578 }}, \"players\": {{ \"max\": 10, \"online\": 7 }}, \"description\": {{ \"text\": \"{}\" }}, \"favicon\": \"data:image/png;base64,{}\"}}", description, icon_b64)
}
