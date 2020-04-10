use std::mem;

use async_std::io::prelude::*;

use async_trait::async_trait;

use crate::error::{McError, McResult};
use crate::field::Field;

// macro_rules! gen_primitive {
//     ($name:ident, $int:ty) => {
//         #[derive(Debug)]
//         pub struct $name($int);
//
//         impl $name {
//             pub fn new(value: $int) -> Self {
//                 Self(value)
//             }
//         }
//
//         #[async_trait]
//         impl Field for $name {
//             type Displayable = $int;
//
//             fn value(&self) -> &Self::Displayable {
//                 &self.0
//             }
//
//             fn size(&self) -> usize {
//                 mem::size_of::<$int>()
//             }
//
//             async fn read_field<R: Read + Unpin + Send>(r: &mut R) -> McResult<Self> {
//                 todo!()
//                 // r.[<read_ $int>]::<BigEndian>() .map_err(McError::Io).map(Self)
//             }
//
//             async fn write_field<W: Write + Unpin + Send>(&self, w: &mut W) -> McResult<()> {
//                 todo!()
//                 // w.[<write_ $int>]::<BigEndian>(self.0).map_err(McError::Io)
//             }
//         }
//     };
// }
//
// gen_primitive!(ShortField, i16);
// gen_primitive!(UShortField, u16);
// gen_primitive!(IntField, i32);
// gen_primitive!(LongField, i64);
// gen_primitive!(FloatField, f32);
// gen_primitive!(DoubleField, f64);

// no endianness for single bytes, and i can't work out how to make this work with the macro
// gen_primitive!(BoolType, bool, i8);
// gen_primitive!(ByteField, i8);
// gen_primitive!(UByteField, u8);

#[derive(Debug)]
pub struct UShortField(u16);

impl UShortField {
    pub fn new(value: u16) -> Self {
        Self(value)
    }
}

#[async_trait]
impl Field for UShortField {
    type Displayable = u16;

    fn value(&self) -> &Self::Displayable {
        &self.0
    }

    fn size(&self) -> usize {
        mem::size_of::<u16>()
    }

    async fn read_field<R: Read + Unpin + Send>(r: &mut R) -> McResult<Self> {
        let mut buf = [0u8; 2];
        r.read_exact(&mut buf).await.map_err(McError::Io)?;
        Ok(unsafe { std::mem::transmute(buf) })
    }

    async fn write_field<W: Write + Unpin + Send>(&self, w: &mut W) -> McResult<()> {
        let buf: [u8; 2] = unsafe { std::mem::transmute(self.0) };
        w.write_all(&buf).await.map_err(McError::Io)
    }
}

// #[derive(Debug)]
// pub struct BoolField(bool);
//
// impl BoolField {
//     pub fn new(value: bool) -> Self {
//         Self(value)
//     }
// }
//
// impl Field for BoolField {
//     type Displayable = bool;
//
//     fn value(&self) -> &Self::Displayable {
//         &self.0
//     }
//
//     fn size(&self) -> usize {
//         mem::size_of::<bool>()
//     }
//
//     fn read<R: Read>(r: &mut R) -> McResult<Self> {
//         r.read_u8().map_err(McError::Io).map(|b| Self(b == 1))
//     }
//
//     fn write<W: Write>(&self, w: &mut W) -> McResult<()> {
//         w.write_u8(self.0 as u8).map_err(McError::Io)
//     }
// }
