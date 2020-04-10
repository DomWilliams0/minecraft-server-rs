use std::fmt::{Display, Formatter};

use crate::field::{Field, VarIntField};
use crate::prelude::*;

pub struct VarIntThenByteArrayField {
    length: VarIntField,
    array: ByteArray,
}

impl VarIntThenByteArrayField {
    pub fn new(buf: Vec<u8>) -> Self {
        Self {
            length: VarIntField::new(buf.len() as i32),
            array: ByteArray(buf),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.array.0
    }
}

#[async_trait]
impl Field for VarIntThenByteArrayField {
    type Displayable = ByteArray;

    fn value(&self) -> &Self::Displayable {
        &self.array
    }

    fn size(&self) -> usize {
        self.length.size() + self.length.value() as usize
    }

    async fn read_field<R: McRead>(r: &mut R) -> McResult<Self> {
        let length = VarIntField::read_field(r).await?;
        let mut array = vec![0u8; length.value() as usize];
        r.read_exact(&mut array).await.map_err(McError::Io)?;

        Ok(Self {
            length,
            array: ByteArray(array),
        })
    }

    async fn write_field<W: McWrite>(&self, w: &mut W) -> McResult<()> {
        self.length.write_field(w).await?;
        w.write_all(&self.array.0).await.map_err(McError::Io)?;
        Ok(())
    }
}

pub struct ByteArray(pub Vec<u8>);

impl Display for ByteArray {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
