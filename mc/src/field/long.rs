use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::error::{McError, McResult};
use crate::field::Field;
use std::mem;

#[derive(Debug)]
pub struct LongField(i64);

impl LongField {
    pub fn new(value: i64) -> Self {
        Self(value)
    }
}

impl Field for LongField {
    type Displayable = i64;

    fn value(&self) -> &Self::Displayable {
        &self.0
    }

    fn size(&self) -> usize {
        mem::size_of::<i64>()
    }

    fn read<R: Read>(r: &mut R) -> McResult<Self> {
        r.read_i64::<BigEndian>().map_err(McError::Io).map(Self)
    }

    fn write<W: Write>(&self, w: &mut W) -> McResult<()> {
        w.write_i64::<BigEndian>(self.0).map_err(McError::Io)
    }
}
