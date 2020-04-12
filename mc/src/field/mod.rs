use std::fmt::Display;
use std::fmt::Formatter;

pub use array::{RestOfPacketByteArrayField, VarIntThenByteArrayField};
pub use primitive::{
    BoolField, ByteField, DoubleField, FloatField, IntField, LongField, ShortField, UByteField,
    UShortField,
};
pub use string::{ChatField, IdentifierField, StringField};
pub use varint::VarIntField;

use crate::prelude::*;

#[async_trait]
pub trait Field: Sized {
    type Displayable: Display;
    fn value(&self) -> &Self::Displayable;

    fn size(&self) -> usize;
    async fn read_field<R: McRead>(r: &mut R) -> McResult<Self>;
    async fn write_field<W: McWrite>(&self, w: &mut W) -> McResult<()>;
}

pub struct DisplayableField<'a, T: Display>(pub &'a T);

impl<'a, T: Display> Display for DisplayableField<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

mod array;
mod primitive;
mod string;
mod varint;

#[cfg(test)]
mod tests {
    use async_std::io::Cursor;

    use crate::field::{Field, StringField, UShortField, VarIntField};

    #[test]
    fn sizes() {
        async_std::task::block_on(async {
            let mut cursor = Cursor::new(vec![0u8, 100]);

            let s = "cor blimey";

            let a = StringField::new(s.to_owned());
            let b = UShortField::new(10);
            let c = VarIntField::new(150);

            let expected_len = 1 + s.len() + 2 + 2;
            assert_eq!(expected_len, a.size() + b.size() + c.size());

            a.write_field(&mut cursor).await.unwrap();
            b.write_field(&mut cursor).await.unwrap();
            c.write_field(&mut cursor).await.unwrap();
            assert_eq!(cursor.position() as usize, expected_len);
        });
    }
}
