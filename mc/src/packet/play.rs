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
#[packet_id = 0x00]
pub struct TeleportConfirm {
    pub teleport_id: VarIntField,
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

#[derive(ClientBoundPacket)]
#[packet_id = 0x3F]
pub struct HeldItemChange {
    pub slot: ByteField,
}

#[derive(ClientBoundPacket)]
#[packet_id = 0x36]
pub struct PlayerPositionAndLook {
    pub x: DoubleField,
    pub y: DoubleField,
    pub z: DoubleField,
    pub yaw: FloatField,
    pub pitch: FloatField,
    pub flags: ByteField,
    pub teleport_id: VarIntField,
}

#[derive(ServerBoundPacket)]
#[packet_id = 0x12]
pub struct PlayerPositionAndRotation {
    pub x: DoubleField,
    pub feet_y: DoubleField,
    pub z: DoubleField,
    pub yaw: FloatField,
    pub pitch: FloatField,
    pub on_ground: BoolField,
}

#[derive(ClientBoundPacket)]
#[packet_id = 0x4E]
pub struct SpawnPosition {
    pub location: PositionField,
}

#[derive(ClientBoundPacket)]
#[packet_id = 0x1B]
pub struct Disconnect {
    pub reason: ChatField,
}

impl PlayerPositionAndLook {
    pub fn new(pos: (f64, f64, f64), teleport_id: i32) -> Self {
        PlayerPositionAndLook {
            x: pos.0.into(),
            y: pos.1.into(),
            z: pos.2.into(),
            yaw: 0.0.into(),
            pitch: 0.0.into(),
            flags: 0.into(), // bitfield, all 0 = all are absolute
            teleport_id: teleport_id.into(),
        }
    }
}

impl Disconnect {
    pub fn with_error(error: &McError) -> Self {
        let msg = if let McError::PleaseDisconnect = error {
            "EOF".to_owned()
        } else {
            format!(
                "§cSHIT, AN ERROR OCCURRED!\n§fpls don't panic\n\n§7{}",
                error
            )
        };

        Disconnect {
            reason: ChatField::new(msg),
        }
    }
}
