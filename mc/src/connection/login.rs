use crate::connection::{ActiveState, LoginState, State};
use crate::error::{McError, McResult};
use crate::field::*;
use crate::packet::*;
use log::*;
use std::io::Write;

impl<W: Write> State<W> for LoginState {
    fn handle_transaction(self, packet: PacketBody, _resp_write: &mut W) -> McResult<ActiveState> {
        match packet.id {
            LoginStart::ID => {
                let login_start = LoginStart::read(packet)?;
                info!("player '{}' is joining", login_start.name.value());

                // TODO get server rsa pub key

                // let enc_req = EncryptionRequest {
                //     server_id: StringField::new("".to_owned()),
                // }
                todo!()
            }
            _ => todo!(),
        }

        todo!()
    }
}
