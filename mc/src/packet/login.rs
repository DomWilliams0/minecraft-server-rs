use async_std::io::Cursor;
use log::*;
use mc_packet_derive::{ClientBoundPacket, ServerBoundPacket};
use std::fmt::{Display, Formatter};

use crate::error::{McError, McResult};
use crate::field::*;
use crate::packet::*;

#[derive(ServerBoundPacket)]
#[packet_id = 0x00]
pub struct LoginStart {
    pub name: StringField, // TODO max length
}

#[derive(ClientBoundPacket)]
#[packet_id = 0x00]
pub struct DisconnectLogin {
    pub reason: ChatField,
}

#[derive(ClientBoundPacket)]
#[packet_id = 0x01]
pub struct EncryptionRequest {
    pub server_id: StringField,
    pub pub_key: VarIntThenByteArrayField,
    pub verify_token: VarIntThenByteArrayField,
}

#[derive(ServerBoundPacket)]
#[packet_id = 0x01]
pub struct EncryptionResponse {
    pub shared_secret: VarIntThenByteArrayField,
    pub verify_token: VarIntThenByteArrayField,
}

#[derive(ClientBoundPacket)]
#[packet_id = 0x02]
pub struct LoginSuccess {
    pub uuid: StringField,
    pub username: StringField,
}

// TODO set compression
