use crate::connection::play::{OfflinePlayState, OnlinePlayState, PlayStateComms};
use crate::connection::{ActiveState, LoginState, PlayState, State};
use crate::error::{McError, McResult};
use crate::field::*;
use crate::packet::*;
use crate::server::{OnlineStatus, ServerDataRef};
use log::*;
use std::io::Write;
use std::mem;
use uuid::adapter::HyphenatedRef;
use uuid::Uuid;

fn generate_verify_token() -> McResult<Vec<u8>> {
    let mut token = vec![0u8; 2];
    openssl::rand::rand_bytes(&mut token).map_err(McError::OpenSSL)?;
    Ok(token)
}

impl<W: Write> State<W> for LoginState {
    fn handle_transaction(
        mut self,
        packet: PacketBody,
        resp_write: &mut W,
        server_data: &ServerDataRef,
    ) -> McResult<ActiveState> {
        match packet.id {
            LoginStart::ID => {
                let login_start = LoginStart::read(packet)?;
                let player_name = login_start.name.take();

                info!("player '{}' is joining", player_name);
                let server_data = server_data.lock().map_err(|_| McError::MutexUnlock)?;
                match server_data.online_status()? {
                    OnlineStatus::Offline => {
                        // no auth
                        let player_uuid = Uuid::new_v4();
                        let (response, play_state) =
                            self.into_play_state(player_name, player_uuid, None)?;

                        response.write(resp_write)?;
                        Ok(ActiveState::Play(play_state))
                    }
                    OnlineStatus::Online { public_key } => {
                        let verify_token = generate_verify_token()?;

                        let enc_req = EncryptionRequest {
                            server_id: StringField::new("".to_owned()),

                            pub_key: VarIntThenByteArrayField::new(public_key),
                            verify_token: VarIntThenByteArrayField::new(verify_token.clone()),
                        };

                        enc_req.write(resp_write)?;

                        let new_state = LoginState {
                            player_name,
                            verify_token,
                        };

                        Ok(ActiveState::Login(new_state))
                    }
                }
            }
            EncryptionResponse::ID => {
                let enc_resp = EncryptionResponse::read(packet)?;

                {
                    let (decrypted_token, decrypted_shared_secret) = {
                        let server_data = server_data.lock().map_err(|_| McError::MutexUnlock)?;
                        let token = server_data.decrypt(&enc_resp.verify_token.bytes())?;
                        let secret = server_data.decrypt(&enc_resp.shared_secret.bytes())?;

                        (token, secret)
                    };

                    // verify token decrypts to the same value
                    if self.verify_token != decrypted_token {
                        warn!(
                            "verify token mismatch: expected {:?} but got {:?}",
                            self.verify_token, decrypted_token
                        );
                        return Err(McError::VerifyTokenMismatch);
                    }

                    let player_name = mem::take(&mut self.player_name);
                    debug!("enabling AES packet encryption for player {}", player_name);

                    // TODO request uuid and authenticate player with mojang
                    let player_uuid = Uuid::new_v4();

                    let (login_success, play_state) = self.into_play_state(
                        player_name,
                        player_uuid,
                        Some(decrypted_shared_secret),
                    )?;

                    login_success.write(resp_write)?;
                    Ok(ActiveState::Play(play_state))
                }
            }
            x => Err(McError::BadPacketId(x)),
        }
    }
}

impl LoginState {
    fn into_play_state(
        self,
        player_name: String,
        player_uuid: Uuid,
        shared_secret: Option<Vec<u8>>,
    ) -> McResult<(LoginSuccess, PlayState)> {
        let encoded_uuid = {
            let mut buf = vec![0u8; HyphenatedRef::LENGTH];
            player_uuid.to_hyphenated_ref().encode_upper(&mut buf);

            // Safety: assuming uuid encodes to valid utf
            unsafe { String::from_utf8_unchecked(buf) }
        };

        let response = LoginSuccess {
            uuid: StringField::new(encoded_uuid),
            username: StringField::new(player_name.clone()),
        };

        let comms: Box<dyn PlayStateComms> = if let Some(shared_secret) = shared_secret {
            Box::new(OnlinePlayState { shared_secret })
        } else {
            Box::new(OfflinePlayState)
        };

        let state = PlayState {
            player_name,
            uuid: player_uuid,
            comms,
        };

        Ok((response, state))
    }
}
