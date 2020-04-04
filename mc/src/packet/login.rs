use std::fmt::{Display, Formatter};
use std::io::{Cursor, Write};

use log::*;

use mc_packet_derive::{ClientBoundPacket, ServerBoundPacket};

use crate::error::{McError, McResult};
use crate::field::*;
use crate::packet::*;

#[derive(ServerBoundPacket)]
#[packet_id = 0x00]
pub struct LoginStart {
    pub name: StringField, // TODO max length
}

#[derive(ServerBoundPacket)]
#[packet_id = 0x00]
pub struct EncryptionRequest {
    pub server_id: StringField,
    pub pub_key: VarIntThenByteArrayField,
    pub verify_token: VarIntThenByteArrayField,
}

#[derive(ClientBoundPacket)]
#[packet_id = 0x01]
pub struct EncryptionResponse {}

#[derive(ClientBoundPacket)]
#[packet_id = 0x01]
pub struct LoginSuccess {}

// TODO set compression
