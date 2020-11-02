use std::convert::TryFrom;

use async_std::io::prelude::*;

use async_trait::async_trait;

use crate::connection::{McRead, McWrite};
use crate::error::{McError, McResult};
use crate::field::{Field, VarIntField};
use std::fmt::{Debug, Formatter};

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

    async fn read_field<R: McRead>(r: &mut R) -> McResult<Self> {
        let length = VarIntField::read_field(r).await?.value() as usize;
        let value = {
            let mut vec = vec![0u8; length];
            r.read_exact(&mut vec).await.map_err(McError::Io)?;
            String::from_utf8(vec)?
        };

        Ok(Self::new(value))
    }

    async fn write_field<W: McWrite>(&self, w: &mut W) -> McResult<()> {
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
    // TODO pass in fmt args somehow to avoid double allocation
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

    async fn read_field<R: McRead>(r: &mut R) -> McResult<Self> {
        Ok(Self {
            string: StringField::read_field(r).await?,
        })
    }

    async fn write_field<W: McWrite>(&self, w: &mut W) -> McResult<()> {
        self.string.write_field(w).await
    }
}

pub struct IdentifierField {
    string: StringField,
    colon: Option<usize>,
}

impl IdentifierField {
    pub fn new(s: String) -> Self {
        let colon = s.find(':');

        Self {
            string: StringField::new(s),
            colon,
        }
    }

    pub fn namespace(&self) -> &str {
        match self.colon {
            Some(idx) => &self.string.value[..idx],
            None => "minecraft",
        }
    }

    pub fn location(&self) -> &str {
        match self.colon {
            Some(idx) => &self.string.value[idx + 1..],
            None => &self.string.value,
        }
    }
}

impl Debug for IdentifierField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace(), self.location())
    }
}

#[async_trait]
impl Field for IdentifierField {
    type Displayable = String;

    fn value(&self) -> &Self::Displayable {
        self.string.value()
    }

    fn size(&self) -> usize {
        self.string.size()
    }

    async fn read_field<R: McRead>(r: &mut R) -> McResult<Self> {
        StringField::read_field(r)
            .await
            .map(|s| Self::new(s.take()))
    }

    async fn write_field<W: McWrite>(&self, w: &mut W) -> McResult<()> {
        self.string.write_field(w).await
    }
}

#[cfg(test)]
mod test {
    use crate::field::string::IdentifierField;

    #[test]
    fn identifier() {
        let default = IdentifierField::new("bonbon".to_owned());
        let custom = IdentifierField::new("colon:sunglass".to_lowercase());
        let bad = IdentifierField::new("ohno:".to_lowercase());

        assert_eq!(default.namespace(), "minecraft");
        assert_eq!(default.location(), "bonbon");

        assert_eq!(custom.namespace(), "colon");
        assert_eq!(custom.location(), "sunglass");

        assert_eq!(bad.namespace(), "ohno");
        assert_eq!(bad.location(), "");
    }
}
