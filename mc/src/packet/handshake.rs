use std::io::Cursor;

use crate::field::DisplayableField;
use log::*;
use std::fmt::{Display, Formatter};

use mc_packet_derive::ServerBoundPacket;

use crate::error::{McError, McResult};
use crate::field::{Field, StringField, UShortField, VarIntField};
use crate::packet::{Packet, PacketBody, PacketId, ServerBound};

#[derive(ServerBoundPacket)]
#[packet_id = 0x00]
pub struct PacketHandshake {
    pub protocol_version: VarIntField,
    pub server_address: StringField,
    pub server_port: UShortField,
    pub next_state: VarIntField,
}
