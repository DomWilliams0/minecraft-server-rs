use std::fmt::{Display, Formatter};

use async_std::io::Cursor;

use mc_packet_derive::{ClientBoundPacket, ServerBoundPacket};

use crate::field::DisplayableField;
use crate::field::*;
use crate::packet::*;
use crate::prelude::*;

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
