use std::convert::TryFrom;

use async_std::io::prelude::WriteExt;
use async_std::io::{Read, Write};
use futures::AsyncReadExt;

use async_trait::async_trait;

use crate::error::{McError, McResult};
use crate::field::{Field, VarIntField};

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

#[async_trait]
impl Field for StringField {
    type Displayable = String;

    fn value(&self) -> &Self::Displayable {
        &self.value
    }

    fn size(&self) -> usize {
        self.length.size() + self.length.value() as usize
    }

    async fn read_field<R: Read + Unpin + Send>(r: &mut R) -> McResult<Self> {
        let length = VarIntField::read_field(r).await?.value() as usize;
        let value = {
            let mut vec = vec![0u8; length];
            r.read_exact(&mut vec).await.map_err(McError::Io)?;
            String::from_utf8(vec).map_err(|_| McError::BadString)?
        };

        Ok(Self::new(value))
    }

    async fn write_field<W: Write + Unpin + Send>(&self, w: &mut W) -> McResult<()> {
        self.length.write_field(w).await?;

        w.write_all(self.value.as_bytes())
            .await
            .map_err(McError::Io)
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

#[async_trait]
impl Field for ChatField {
    type Displayable = String;

    fn value(&self) -> &Self::Displayable {
        &self.string.value()
    }

    fn size(&self) -> usize {
        self.string.size()
    }

    async fn read_field<R: Read + Unpin + Send>(r: &mut R) -> McResult<Self> {
        Ok(Self {
            string: StringField::read_field(r).await?,
        })
    }

    async fn write_field<W: Write + Unpin + Send>(&self, w: &mut W) -> McResult<()> {
        self.string.write_field(w).await
    }
}
