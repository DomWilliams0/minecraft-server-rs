use std::io;
use thiserror::*;

use crate::packet::PacketId;
use futures::channel::mpsc::SendError;
use std::string::FromUtf8Error;

use crate::game::ClientUuid;

pub type McResult<T> = Result<T, McError>;

// TODO simplify map_err
#[derive(Debug, Error)]
pub enum McError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Sink error: {0}")]
    Sink(#[from] SendError),

    #[error("Unknown sink error")]
    SinkUnknown,

    #[error("Bad bool value, must be 0 or 1 (got {0})")]
    BadBool(u8),

    #[error("Varint is longer than the max of 5 bytes (got {0} bytes)")]
    BadVarInt(usize),

    #[error("Invalid packet length {0}")]
    BadPacketLength(usize),

    #[error("Invalid next state {0}")]
    BadNextState(i32),

    // used in macros
    #[error("Expected packet ID {expected:#x} but got {actual:#x}")]
    UnexpectedPacket {
        expected: PacketId,
        actual: PacketId,
    },

    // used in macros
    #[error("Failed to read packet of length {length}, read {read} bytes")]
    FullPacketNotRead { length: usize, read: usize },

    #[error("Invalid unicode string: {0}")]
    BadString(#[from] FromUtf8Error),

    #[error("Unknown packet ID {0:#x}")]
    BadPacketId(PacketId),

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
}
