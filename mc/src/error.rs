use std::io;
use thiserror::*;

use futures::channel::mpsc::SendError;

use crate::game::ClientUuid;

pub type McResult<T> = Result<T, McError>;

#[derive(Debug, Error)]
pub enum McError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Sink(#[from] SendError),

    #[error(transparent)]
    Packet(#[from] packets::types::PacketError),

    #[error("Unknown sink error")]
    SinkUnknown,

    #[error("Malformed packet with length {0}")]
    MalformedPacket(usize),

    #[error("Unexpected packet with ID {0:#04x}")]
    BadPacketId(i32),

    #[error("Orderly disconnection, have a nice day")]
    PleaseDisconnect,

    #[error("OpenSSL error: {0}")]
    OpenSSL(#[from] openssl::error::ErrorStack),

    #[error("Verify token mismatch")]
    VerifyTokenMismatch,

    #[error("Invalid Mojang authentication response: {0}")]
    Auth(#[source] io::Error),

    #[error("Unexpected JSON response from Mojang during authentication")]
    BadAuthResponse,

    #[error("Unexpected {0} response from Mojang during authentication")]
    UnexpectedAuthResponse(u16),

    #[error("No such player with UUID {0:?}")]
    NoSuchPlayer(ClientUuid),

    #[error("Incorrect teleport confirmation, expected {expected:?} but got {actual}")]
    IncorrectTeleportConfirm { expected: Option<i32>, actual: i32 },

    #[error("Incorrect keep-alive response, expected 4 but got {0}")]
    IncorrectKeepAlive(i64),

    #[error("Invalid next state {0}")]
    BadNextState(i32),
}
