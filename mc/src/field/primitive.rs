use std::mem;

use async_std::io::prelude::*;

use async_trait::async_trait;

use crate::connection::{McRead, McWrite};
use crate::error::{McError, McResult};
use crate::field::Field;

macro_rules! gen_primitive {
    // tt because of https://github.com/dtolnay/async-trait/issues/46
    ($name:tt, $int:tt, $tests:ident) => {
        #[derive(Debug, Clone)]
        pub struct $name($int);

        impl $name {
            pub fn new(value: $int) -> Self {
                Self(value)
            }
        }

        #[async_trait]
        impl Field for $name {
            type Displayable = $int;

            fn value(&self) -> &Self::Displayable {
                &self.0
            }

            fn size(&self) -> usize {
                mem::size_of::<$int>()
            }

            async fn read_field<R: McRead>(r: &mut R) -> McResult<$name> {
                let mut buf = [0u8; mem::size_of::<$int>()];
                r.read_exact(&mut buf).await.map_err(McError::Io)?;

                let val = $int::from_be_bytes(buf);
                Ok(Self(val))
            }

            async fn write_field<W: McWrite>(&self, w: &mut W) -> McResult<()> {
                let buf = $int::to_be_bytes(self.0);
                w.write_all(&buf).await.map_err(McError::Io)
            }
        }

        impl From<$int> for $name {
            fn from(i: $int) -> $name {
                $name::new(i)
            }
        }

        #[cfg(test)]
        mod $tests {
            use super::*;
            use async_std::io::{Cursor, SeekFrom};
            use quickcheck::{Arbitrary, Gen};
            use quickcheck_macros::quickcheck;

            impl Arbitrary for $name {
                fn arbitrary<G: Gen>(g: &mut G) -> Self {
                    $name::new($int::arbitrary(g))
                }
            }

            #[quickcheck]
            fn values(field: $name) {
                async_std::task::block_on(async {
                    let mut cursor = Cursor::new(vec![0u8; mem::size_of::<$int>()]);
                    field.write_field(&mut cursor).await.unwrap();

                    assert_eq!(cursor.position(), field.size() as u64);
                    cursor.seek(SeekFrom::Start(0)).await.unwrap();

                    let read = $name::read_field(&mut cursor).await.unwrap();
                    assert_eq!(cursor.position(), field.size() as u64);
                    assert_eq!(read.value(), field.value());
                });
            }
        }
    };
}

gen_primitive!(ShortField, i16, short);
gen_primitive!(UShortField, u16, ushort);
gen_primitive!(IntField, i32, int);
gen_primitive!(LongField, i64, long);
gen_primitive!(FloatField, f32, float);
gen_primitive!(DoubleField, f64, double);
gen_primitive!(ByteField, i8, byte);
gen_primitive!(UByteField, u8, ubyte);

// bool is a special one, only 1 byte
#[derive(Debug, Clone)]
pub struct BoolField(bool);

impl BoolField {
    pub fn new(value: bool) -> Self {
        Self(value)
    }
}

#[async_trait]
impl Field for BoolField {
    type Displayable = bool;

    fn value(&self) -> &Self::Displayable {
        &self.0
    }

    fn size(&self) -> usize {
        1
    }

    async fn read_field<R: McRead>(r: &mut R) -> McResult<Self> {
        let mut buf = [0u8; 1];
        r.read_exact(&mut buf).await.map_err(McError::Io)?;

        match buf[0] {
            0 => Ok(Self(false)),
            1 => Ok(Self(true)),
            _ => Err(McError::BadBool),
        }
    }

    async fn write_field<W: McWrite>(&self, w: &mut W) -> McResult<()> {
        let buf = [self.0 as u8];
        w.write_all(&buf).await.map_err(McError::Io)
    }
}

impl From<bool> for BoolField {
    fn from(b: bool) -> Self {
        Self::new(b)
    }
}

#[cfg(test)]
mod bool {
    use async_std::io::{Cursor, SeekFrom};
    use quickcheck_macros::quickcheck;

    use quickcheck::{Arbitrary, Gen};

    use super::*;

    impl Arbitrary for BoolField {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            BoolField::new(bool::arbitrary(g))
        }
    }

    #[quickcheck]
    fn values(field: BoolField) {
        async_std::task::block_on(async {
            let mut cursor = Cursor::new(vec![0u8; mem::size_of::<bool>()]);
            field.write_field(&mut cursor).await.unwrap();

            assert_eq!(cursor.position(), field.size() as u64);
            cursor.seek(SeekFrom::Start(0)).await.unwrap();

            let read = BoolField::read_field(&mut cursor).await.unwrap();
            assert_eq!(cursor.position(), field.size() as u64);
            assert_eq!(read.value(), field.value());
        });
    }

    #[test]
    fn invalid_bool_values() {
        async_std::task::block_on(async {
            let mut cursor = Cursor::new(vec![5u8]);
            let read = BoolField::read_field(&mut cursor).await;
            assert!(read.is_err())
        });
    }
}
