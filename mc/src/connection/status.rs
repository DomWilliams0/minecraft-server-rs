use crate::connection::{ActiveState, State, StatusState};
use crate::error::{McError, McResult};
use crate::field::*;
use crate::packet::*;
use std::io::Write;

impl<W: Write> State<W> for StatusState {
    fn handle_transaction(self, packet: PacketBody, resp_write: &mut W) -> McResult<ActiveState> {
        match packet.id {
            PacketEmpty::ID => {
                let _empty = PacketEmpty::read(packet)?;

                let response = PacketStatusResponse {
                    json_response: StringField::new(generate_json(
                        "mInEcRaFt",
                        include_str!("../../icon.png.base64"),
                    )),
                };

                response.write(resp_write)?;
                Ok(ActiveState::Status(self))
            }
            PacketPing::ID => {
                let ping = PacketPing::read(packet)?;
                let response = PacketPong {
                    payload: ping.payload,
                };

                response.write(resp_write)?;
                Err(McError::PleaseDisconnect)
            }
            x => Err(McError::BadPacketId(x)),
        }
    }
}

fn generate_json(description: &str, icon_b64: &str) -> String {
    format!("{{\"version\": {{\"name\": \"1.15.2\", \"protocol\": 578 }}, \"players\": {{ \"max\": 10, \"online\": 7 }}, \"description\": {{ \"text\": \"{}\" }}, \"favicon\": \"data:image/png;base64,{}\"}}", description, icon_b64)
}
