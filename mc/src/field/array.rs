use crate::error::{McError, McResult};
use crate::field::*;
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};

pub struct VarIntThenByteArrayField {
    length: VarIntField,
    array: ByteArray,
}

impl Field for VarIntThenByteArrayField {
    type Displayable = ByteArray;

    fn value(&self) -> &Self::Displayable {
        &self.array
    }

    fn size(&self) -> usize {
        self.length.value() as usize
    }

    fn read<R: Read>(r: &mut R) -> McResult<Self> {
        let length = VarIntField::read(r)?;
        let mut array = vec![0u8; length.value() as usize];
        r.read_exact(&mut array).map_err(McError::Io)?;

        Ok(Self {
            length,
            array: ByteArray(array),
        })
    }

    fn write<W: Write>(&self, w: &mut W) -> McResult<()> {
        self.length.write(w)?;
        w.write_all(&self.array.0).map_err(McError::Io)?;
        Ok(())
    }
}

pub struct ByteArray(pub Vec<u8>);

impl Display for ByteArray {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
