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

#[derive(ServerBoundPacket)]
#[packet_id = 0x05]
pub struct ClientSettings {
    pub locale: StringField,
    pub view_distance: ByteField,
    pub chat_mode: VarIntField,
    pub chat_colors: BoolField,
    pub displayed_skin_parts: UByteField,
    pub main_hand: VarIntField,
}

#[derive(ServerBoundPacket)]
#[packet_id = 0x0B]
pub struct PluginMessage {
    pub channel: IdentifierField,
    pub data: RestOfPacketByteArrayField,
}
