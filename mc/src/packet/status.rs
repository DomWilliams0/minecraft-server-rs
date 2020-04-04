use std::io::{Cursor, Write};

use crate::field::DisplayableField;
use log::*;

use mc_packet_derive::{ClientBoundPacket, ServerBoundPacket};

use crate::error::{McError, McResult};
use crate::field::*;
use crate::packet::*;
use std::fmt::{Display, Formatter};

#[derive(ServerBoundPacket)]
#[packet_id = 0x00]
pub struct PacketEmpty;

#[derive(ServerBoundPacket)]
#[packet_id = 0x01]
pub struct PacketPing {
    pub payload: LongField,
}

// ---

#[derive(ClientBoundPacket)]
#[packet_id = 0x00]
pub struct PacketStatusResponse {
    pub json_response: StringField,
}

#[derive(ClientBoundPacket)]
#[packet_id = 0x01]
pub struct PacketPong {
    pub payload: LongField,
}
