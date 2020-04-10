use std::fmt::{Display, Formatter};

use async_std::io::Cursor;
use log::*;

use mc_packet_derive::ServerBoundPacket;

use crate::error::{McError, McResult};
use crate::field::DisplayableField;
use crate::field::*;
use crate::packet::*;

#[derive(ServerBoundPacket)]
#[packet_id = 0x00]
pub struct Handshake {
    pub protocol_version: VarIntField,
    pub server_address: StringField,
    pub server_port: UShortField,
    pub next_state: VarIntField,
}
