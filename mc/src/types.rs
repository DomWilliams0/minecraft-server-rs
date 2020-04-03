use crate::error::{McError, McResult};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};
use std::ops::BitAnd;

pub struct VarInt(i32);

impl VarInt {
    pub fn read<R: Read>(r: &mut R) -> McResult<Self> {
        let mut out = 0u32;
        let mut n = 0;

        loop {
            let byte = r.read_u8().map_err(McError::Io)?;
            out |= (((byte & 0x7f) as u32) << (7 * n)) as u32;
            n += 1;

            if byte.bitand(0x80) == 0 {
                break;
            }
        }

        if n > 5 {
            Err(McError::BadVarInt)
        } else {
            let val = unsafe { std::mem::transmute(out) };
            Ok(Self(val))
        }
    }

    pub fn new(val: i32) -> Self {
        Self(val)
    }

    pub fn write<W: Write>(&self, w: &mut W) -> McResult<()> {
        let mut n = 0;
        // let mut val: u32 = unsafe{std::mem::transmute(self.0)};
        let mut val: u32 = self.0 as u32;

        loop {
            let mut next: u8 = (val & 0x7f) as u8;
            val >>= 7;

            if val > 0 {
                next |= 0x80;
            }

            w.write_u8(next).map_err(McError::Io)?;

            n += 1;
            if val == 0 {
                break;
            }
        }

        if n > 5 {
            Err(McError::BadVarInt)
        } else {
            Ok(())
        }
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::types::VarInt;
    use std::io::Cursor;

    fn assert_varint(val: i32, bytes: &[u8]) {
        let varint = VarInt::new(val);
        assert_eq!(varint.value(), val);

        // encode to bytes
        let mut cursor = Cursor::new(vec![0u8; 5]);
        varint.write(&mut cursor).unwrap();
        let encoded = cursor.into_inner();

        bytes.iter().zip(&encoded).for_each(|(a, b)| {
            assert_eq!(*a, *b);
        });

        // decode back to int
        let mut cursor = Cursor::new(encoded);
        let decoded = VarInt::read(&mut cursor).unwrap();
        assert_eq!(decoded.value(), val);
        bytes
            .iter()
            .zip(&cursor.into_inner())
            .for_each(|(a, b)| assert_eq!(*a, *b));
    }

    #[test]
    fn varint() {
        assert_varint(0, &[0x00]);
        assert_varint(1, &[0x01]);
        assert_varint(127, &[0x7f]);
        assert_varint(128, &[0x80, 0x01]);
        assert_varint(255, &[0xff, 0x01]);
        assert_varint(2_147_483_647, &[0xff, 0xff, 0xff, 0xff, 0x07]);
        assert_varint(-1, &[0xff, 0xff, 0xff, 0xff, 0x0f]);
        assert_varint(-2_147_483_648, &[0x80, 0x80, 0x80, 0x80, 0x08]);
    }
}
