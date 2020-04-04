use std::io;

use crate::packet::PacketId;

pub type McResult<T> = Result<T, McError>;

#[derive(Debug)]
pub enum McError {
    Io(io::Error),
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
    MutexUnlock,
}
