use std::fmt::{Display, Formatter};
use std::io;

use crate::packet::PacketId;

pub type McResult<T> = Result<T, McError>;

#[derive(Debug)]
pub enum McError {
    Io(io::Error),
    ResponseSink,
    BadBool,
    StreamFlush(String),
    BadVarInt,
    BadPacketLength(usize),
    BadNextState(i32),
    BadString,
    UnexpectedPacket {
        expected: PacketId,
        actual: PacketId,
    },
    BadPacketId(PacketId),
    FullPacketNotRead {
        length: usize,
        read: usize,
    },
    PleaseDisconnect,
    NotImplemented,
    MutexUnlock,
    OpenSSL(openssl::error::ErrorStack),
    MissingClientData,
    BadClientData,
    VerifyTokenMismatch,
    Auth(io::Error),
    BadAuthResponse,
    UnexpectedAuthResponse(u16),
}

impl Display for McError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self) // TODO
    }
}
