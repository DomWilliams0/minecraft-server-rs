use std::io::Read;

use byteorder::{BigEndian, ReadBytesExt};

use crate::error::{McError, McResult};
use crate::field::Field;

#[repr(transparent)]
#[derive(Debug)]
pub struct UShortField(u16);

impl Field for UShortField {
    fn read<R: Read>(r: &mut R) -> McResult<Self> {
        r.read_u16::<BigEndian>().map_err(McError::Io).map(Self)
    }
}
