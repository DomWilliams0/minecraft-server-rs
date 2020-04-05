use crate::error::{McError, McResult};
use crate::field::Field;
use byteorder::ReadBytesExt;
use std::io::{Read, Write};
use std::ops::BitAnd;

#[derive(Debug, Copy, Clone)]
pub struct VarIntField {
    value: i32,
    bytes: [u8; 5],
    byte_count: u8,
}

impl Field for VarIntField {
    type Displayable = i32;

    fn value(&self) -> &Self::Displayable {
        &self.value
    }

    fn size(&self) -> usize {
        self.byte_count as usize
    }

    fn read<R: Read>(r: &mut R) -> McResult<Self> {
        let mut out = 0u32;
        let mut n = 0;
        let mut bytes = [0u8; 5];

        loop {
            let byte = r.read_u8().map_err(McError::Io)?;
            bytes[n] = byte;

            out |= ((byte & 0x7f) as u32) << (7 * n as u32);
            n += 1;

            if byte.bitand(0x80) == 0 {
                break;
            }
        }

        if n > 5 {
            Err(McError::BadVarInt)
        } else {
            let value = unsafe { std::mem::transmute(out) };
            Ok(Self {
                value,
                bytes,
                byte_count: n as u8,
            })
        }
    }
    fn write<W: Write>(&self, w: &mut W) -> McResult<()> {
        w.write_all(self.bytes()).map_err(McError::Io)
    }
}

impl VarIntField {
    pub fn new(value: i32) -> Self {
        let mut n = 0;
        let mut val = value as u32;
        let mut bytes = [0u8; 5];

        loop {
            let mut next: u8 = (val & 0x7f) as u8;

            val >>= 7;
            if val > 0 {
                next |= 0x80;
            }

            bytes[n] = next;

            n += 1;
            if val == 0 {
                break;
            }
        }

        assert!(n >= 1 && n <= 5, "somehow i32 is bigger than i32");
        Self {
            value,
            bytes,
            byte_count: n as u8,
        }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..self.byte_count as usize]
    }
}

#[cfg(test)]
mod tests {
    use crate::field::*;
    use std::io::Cursor;

    fn assert_varint(val: i32, bytes: &[u8]) {
        let varint = VarIntField::new(val);
        assert_eq!(varint.value(), val);
        assert_eq!(varint.bytes(), bytes);

        // encode to bytes
        let mut cursor = Cursor::new(vec![0u8; 5]);
        varint.write(&mut cursor).unwrap();
        let encoded = cursor.into_inner();

        bytes.iter().zip(&encoded).for_each(|(a, b)| {
            assert_eq!(*a, *b);
        });

        // decode back to int
        let mut cursor = Cursor::new(encoded);
        let decoded = VarIntField::read(&mut cursor).unwrap();
        assert_eq!(decoded.value(), val);
        assert_eq!(decoded.bytes(), bytes);
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
    #[test]
    fn varint_stream() {
        let mut stream = Cursor::new(vec![1u8, 1u8, 1u8]);
        let varints: Vec<i32> = (0..3)
            .map(|i| {
                VarIntField::read(&mut stream)
                    .unwrap_or_else(|_| panic!("failed varint #{}", i))
                    .value()
            })
            .collect();
        assert_eq!(varints, vec![1, 1, 1]);
    }
}