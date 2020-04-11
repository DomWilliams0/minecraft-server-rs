use std::mem;

use uuid::adapter::HyphenatedRef;
use uuid::Uuid;

use crate::connection::comms::{CommsRef, ResponseSink};
use crate::connection::{ActiveState, LoginState, PlayState, State};
use crate::field::*;
use crate::packet::*;
use crate::prelude::*;
use crate::server::{OnlineStatus, ServerData};

fn generate_verify_token() -> McResult<Vec<u8>> {
    let mut token = vec![0u8; 2];
    openssl::rand::rand_bytes(&mut token).map_err(McError::OpenSSL)?;
    Ok(token)
}

const SERVER_ID: &str = "";

#[async_trait]
impl<R: ResponseSink> State<R> for LoginState {
    async fn handle_transaction(
        mut self,
        packet: PacketBody,
        server_data: &ServerData,
        comms: &mut CommsRef<R>,
    ) -> McResult<ActiveState> {
        let result = async {
            match packet.id {
                LoginStart::ID => {
                    let login_start = LoginStart::read_packet(packet).await?;
                    let player_name = login_start.name.take();

                    info!("player '{}' is joining", player_name);
                    match server_data.online_status()? {
                        OnlineStatus::Offline => {
                            // no auth
                            let player_uuid = Uuid::new_v4();
                            let (login_success, play_state) =
                                self.into_play_state(player_name, player_uuid)?;

                            comms.send_response(login_success).await?;
                            Ok(ActiveState::Play(play_state))
                        }
                        OnlineStatus::Online { public_key } => {
                            let verify_token = generate_verify_token()?;

                            let enc_req = EncryptionRequest {
                                server_id: StringField::new(SERVER_ID.to_owned()),

                                pub_key: VarIntThenByteArrayField::new(public_key),
                                verify_token: VarIntThenByteArrayField::new(verify_token.clone()),
                            };

                            comms.send_response(enc_req).await?;

                            let new_state = LoginState {
                                player_name,
                                verify_token,
                            };

                            Ok(ActiveState::Login(new_state))
                        }
                    }
                }
                EncryptionResponse::ID => {
                    let enc_resp = EncryptionResponse::read_packet(packet).await?;

                    {
                        let (decrypted_token, decrypted_shared_secret, public_key) = {
                            let token = server_data.decrypt(&enc_resp.verify_token.bytes())?;
                            let secret = server_data.decrypt(&enc_resp.shared_secret.bytes())?;
                            let public_key = server_data.public_key()?;

                            (token, secret, public_key)
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
                        comms.upgrade(decrypted_shared_secret.clone()).await;

                        debug!("authenticating player with mojang");
                        let auth_response = auth::auth(
                            &player_name,
                            SERVER_ID,
                            &decrypted_shared_secret,
                            &public_key,
                        )?;
                        debug!(
                            "authenticated player with mojang, got uuid of {}",
                            auth_response.uuid
                        );

                        let (login_success, play_state) =
                            self.into_play_state(player_name, auth_response.uuid)?;

                        comms.send_response(login_success).await?;
                        Ok(ActiveState::Play(play_state))
                    }
                }
                x => Err(McError::BadPacketId(x)),
            }
        }
        .await;

        if let Err(e) = &result {
            let disconnect = Disconnect {
                reason: ChatField::new(format!("Error: {}", e)),
            };

            // ignore error
            let _ = comms.send_response(disconnect).await;
        }

        result
    }
}

impl LoginState {
    fn into_play_state(
        self,
        player_name: String,
        player_uuid: Uuid,
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

        let state = PlayState {
            player_name,
            uuid: player_uuid,
        };

        Ok((response, state))
    }
}

mod auth {
    use std::str::FromStr;

    use log::*;
    use num::BigInt;
    use openssl::sha::Sha1;
    use uuid::Uuid;

    use crate::error::{McError, McResult};

    pub struct AuthResponse {
        pub uuid: Uuid,
        // TODO skin
    }

    fn generate_hash(server_id: &str, shared_secret: &[u8], public_key: &[u8]) -> String {
        let mut sha1 = Sha1::new();
        sha1.update(server_id.as_bytes());
        sha1.update(shared_secret);
        sha1.update(public_key);
        finish_hash(sha1)
    }

    fn finish_hash(sha: Sha1) -> String {
        let raw_bytes = sha.finish();
        let big_int = BigInt::from_signed_bytes_be(&raw_bytes);

        big_int.to_str_radix(16)
    }

    pub fn auth(
        player_name: &str,
        server_id: &str,
        shared_secret: &[u8],
        public_key: &[u8],
    ) -> McResult<AuthResponse> {
        let hash = generate_hash(server_id, shared_secret, public_key);

        let response = ureq::get("https://sessionserver.mojang.com/session/minecraft/hasJoined")
            .query("username", player_name)
            .query("serverId", &hash)
            .timeout_connect(10_000)
            .timeout_read(10_000)
            .call();

        if response.ok() {
            let json = match response.into_json().map_err(McError::Auth)? {
                ureq::SerdeValue::Object(obj) => obj,
                _ => return Err(McError::BadAuthResponse),
            };

            match (json.get("id"), json.get("name")) {
                (Some(ureq::SerdeValue::String(uuid)), Some(ureq::SerdeValue::String(name))) => {
                    if name != player_name {
                        warn!("incorrect player name returned from mojang, expected '{}' but got '{}'", player_name, name);
                        return Err(McError::BadAuthResponse);
                    }

                    let uuid = match Uuid::from_str(uuid) {
                        Ok(uuid) => uuid,
                        Err(_) => {
                            warn!("bad uuid returned from mojang ('{}')", uuid);
                            return Err(McError::BadAuthResponse);
                        }
                    };

                    Ok(AuthResponse { uuid })
                }
                _ => {
                    warn!("bad json returned from mojang");
                    Err(McError::BadAuthResponse)
                }
            }
        } else {
            Err(McError::UnexpectedAuthResponse(response.status()))
        }
    }

    #[cfg(test)]
    fn generate_hash_from_name(name: &str) -> String {
        let mut sha1 = Sha1::new();
        sha1.update(name.as_bytes());
        finish_hash(sha1)
    }

    #[cfg(test)]
    mod tests {
        use super::generate_hash_from_name;

        #[test]
        fn hash() {
            assert_eq!(
                "-7c9d5b0044c130109a5d7b5fb5c317c02b4e28c1",
                generate_hash_from_name("jeb_")
            );
            assert_eq!(
                "4ed1f46bbe04bc756bcb17c0c7ce3e4632f06a48",
                generate_hash_from_name("Notch")
            );
            assert_eq!(
                "88e16a1019277b15d58faf0541e11910eb756f6",
                generate_hash_from_name("simon")
            );
        }
    }
}
