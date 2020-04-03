use crate::error::{McError, McResult};
use crate::field::{Field, VarIntField};
use std::io::Read;

#[derive(Debug)]
pub struct StringField(String);

impl Field for StringField {
    fn read<R: Read>(r: &mut R) -> McResult<Self> {
        let length = VarIntField::read(r)?.value() as usize;
        let value = {
            let mut vec = vec![0u8; length];
            r.read_exact(&mut vec).map_err(McError::Io)?;
            String::from_utf8(vec).map_err(|_| McError::BadString)?
        };

        Ok(Self(value))
    }
}
