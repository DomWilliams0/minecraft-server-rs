use crate::error::McResult;
use std::io::Read;

pub trait Field: Sized {
    fn size() -> usize {
        std::mem::size_of::<Self>()
    }
    fn read<R: Read>(r: &mut R) -> McResult<Self>;
}

mod string;
mod ushort;
mod varint;

pub use string::StringField;
pub use ushort::UShortField;
pub use varint::VarIntField;
