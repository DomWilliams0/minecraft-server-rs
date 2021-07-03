use packets::v1_15_2 as mc;

use crate::error::McError;
pub use mc::*;
use packets::types::StringField;
use std::borrow::Cow;

pub trait DisconnectExt: Sized {
    fn with_error(error: &McError) -> Self;
}

pub trait KeepAliveExt: Sized {
    fn generate() -> Self;
}

pub trait PlayerPositionAndLookExt: Sized {
    fn new(pos: (f64, f64, f64), teleport_id: i32) -> Self;
}

fn disconnect_reason(error: &McError) -> StringField {
    let reason = if let McError::PleaseDisconnect = error {
        Cow::Borrowed("EOF")
    } else {
        Cow::Owned(format!(
            "§cSHIT, AN ERROR OCCURRED!\n§fpls don't panic\n\n§7{}",
            error
        ))
    };
    StringField::new_chat(reason)
}
impl DisconnectExt for login::client::Disconnect {
    fn with_error(error: &McError) -> Self {
        Self {
            reason: disconnect_reason(error),
        }
    }
}

impl DisconnectExt for play::client::KickDisconnect {
    fn with_error(error: &McError) -> Self {
        Self {
            reason: disconnect_reason(error),
        }
    }
}

impl KeepAliveExt for play::client::KeepAlive {
    fn generate() -> Self {
        let random = 4; // good enough
        Self {
            keep_alive_id: random.into(),
        }
    }
}

impl PlayerPositionAndLookExt for play::client::Position {
    fn new(pos: (f64, f64, f64), teleport_id: i32) -> Self {
        Self {
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
