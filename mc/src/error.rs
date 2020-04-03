use std::io;

pub type McResult<T> = Result<T, McError>;

#[derive(Debug)]
pub enum McError {
    Io(io::Error),
    BadVarInt,
    BadPacketLength(usize),
    BadNextState(i32),
}
