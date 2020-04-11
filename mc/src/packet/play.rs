use log::*;
use mc_packet_derive::{ClientBoundPacket, ServerBoundPacket};
use std::fmt::{Display, Formatter};

use crate::error::McResult;
use crate::field::*;
use crate::packet::*;

#[derive(ClientBoundPacket)]
#[packet_id = 0x26]
pub struct JoinGame {
    pub entity_id: IntField,
    pub gamemode: UByteField,
    pub dimension: IntField,
    pub hashed_seed: LongField,
    pub max_players: UByteField,
    pub level_type: StringField,
    pub view_distance: VarIntField,
    pub reduced_debug_info: BoolField,
    pub enable_respawn_screen: BoolField,
}
