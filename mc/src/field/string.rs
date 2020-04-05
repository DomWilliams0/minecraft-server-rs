use crate::error::{McError, McResult};
use crate::field::*;
use std::convert::TryFrom;
use std::io::{Read, Write};

#[derive(Debug)]
pub struct StringField {
    value: String,

    /// String length
    length: VarIntField,
}

impl StringField {
    pub fn new(value: String) -> Self {
        assert!(i32::try_from(value.len()).is_ok());

        let len = value.len();
        Self {
            value,
            length: VarIntField::new(len as i32),
        }
    }

    pub fn take(self) -> String {
        self.value
    }
}

impl Field for StringField {
    type Displayable = String;

    fn value(&self) -> &Self::Displayable {
        &self.value
    }

    fn size(&self) -> usize {
        self.length.size() + self.length.value() as usize
    }

    fn read<R: Read>(r: &mut R) -> McResult<Self> {
        let length = VarIntField::read(r)?.value() as usize;
        let value = {
            let mut vec = vec![0u8; length];
            r.read_exact(&mut vec).map_err(McError::Io)?;
            String::from_utf8(vec).map_err(|_| McError::BadString)?
        };

        Ok(Self::new(value))
    }

    fn write<W: Write>(&self, w: &mut W) -> McResult<()> {
        self.length.write(w)?;

        w.write_all(self.value.as_bytes()).map_err(McError::Io)
    }
}

pub struct ChatField {
    // TODO colors and stuff
    string: StringField,
}

impl ChatField {
    pub fn new(value: String) -> Self {
        Self {
            string: StringField::new(format!(r#"{{"text": "{}"}}"#, value)),
        }
    }
}

impl Field for ChatField {
    type Displayable = String;

    fn value(&self) -> &Self::Displayable {
        &self.string.value()
    }

    fn size(&self) -> usize {
        self.string.size()
    }

    fn read<R: Read>(r: &mut R) -> McResult<Self> {
        Ok(Self {
            string: StringField::read(r)?,
        })
    }

    fn write<W: Write>(&self, w: &mut W) -> McResult<()> {
        self.string.write(w)
    }
}
