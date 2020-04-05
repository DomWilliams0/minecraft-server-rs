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
pub struct Empty;

#[derive(ServerBoundPacket)]
#[packet_id = 0x01]
pub struct Ping {
    pub payload: LongField,
}

// ---

#[derive(ClientBoundPacket)]
#[packet_id = 0x00]
pub struct StatusResponse {
    pub json_response: StringField,
}

#[derive(ClientBoundPacket)]
#[packet_id = 0x01]
pub struct Pong {
    pub payload: LongField,
}
