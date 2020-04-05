use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::error::{McError, McResult};
use crate::field::Field;
use std::mem;

#[derive(Debug)]
pub struct UShortField(u16);

impl UShortField {
    pub fn new(value: u16) -> Self {
        Self(value)
    }
}

impl Field for UShortField {
    type Displayable = u16;

    fn value(&self) -> &Self::Displayable {
        &self.0
    }

    fn size(&self) -> usize {
        mem::size_of::<u16>()
    }

    fn read<R: Read>(r: &mut R) -> McResult<Self> {
        r.read_u16::<BigEndian>().map_err(McError::Io).map(Self)
    }

    fn write<W: Write>(&self, w: &mut W) -> McResult<()> {
        w.write_u16::<BigEndian>(self.0).map_err(McError::Io)
    }
}