use std::fmt::Display;
use std::fmt::Formatter;
use std::io::{Read, Write};

use crate::error::McResult;

pub trait Field: Sized {
    type Displayable: Display;
    fn value(&self) -> &Self::Displayable;

    fn size(&self) -> usize;
    fn read<R: Read>(r: &mut R) -> McResult<Self>;
    fn write<W: Write>(&self, w: &mut W) -> McResult<()>;
}

pub struct DisplayableField<'a, T: Display>(pub &'a T);

impl<'a, T: Display> Display for DisplayableField<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

mod array;
mod long;
mod string;
mod ushort;
mod varint;

pub use array::VarIntThenByteArrayField;
pub use long::LongField;
pub use string::StringField;
pub use ushort::UShortField;
pub use varint::VarIntField;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn sizes() {
        let mut cursor = Cursor::new(vec![0u8, 100]);

        let s = "cor blimey";

        let a = StringField::new(s.to_owned());
        let b = UShortField::new(10);
        let c = VarIntField::new(150);

        let expected_len = 1 + s.len() + 2 + 2;
        assert_eq!(expected_len, a.size() + b.size() + c.size());

        a.write(&mut cursor).unwrap();
        b.write(&mut cursor).unwrap();
        c.write(&mut cursor).unwrap();
        assert_eq!(cursor.position() as usize, expected_len);
    }
}
