use crate::error::McResult;
use crate::field::{Field, LongField};

use crate::prelude::*;
use async_trait::async_trait;
use std::fmt::{Display, Formatter};

// TODO position field impl is probably wrong
#[derive(Debug)]
pub struct PositionField {
    x: i32,
    y: i32,
    z: i32,
}

#[async_trait]
impl Field for PositionField {
    type Displayable = Self;

    fn value(&self) -> &Self::Displayable {
        self
    }

    fn size(&self) -> usize {
        8
    }

    async fn read_field<R: McRead>(r: &mut R) -> McResult<Self> {
        let long = LongField::read_field(r).await?;
        // let long = u64::from_ne_bytes(long.value().to_ne_bytes());
        let long = *long.value();

        let x = (long >> 38) as i64;
        let y = (long & 0xFFF) as i64;
        let z = (long << 26 >> 38) as i64;

        // if x >= 2u64.pow(25) {
        //     x -= 2u64.pow(26)
        // }
        // if y >= 2u64.pow(11) {
        //     y -= 2u64.pow(12)
        // }
        // if z >= 2u64.pow(25) {
        //     z -= 2u64.pow(26)
        // }

        Ok(PositionField {
            x: x as i32,
            y: y as i32,
            z: z as i32,
        })
    }

    async fn write_field<W: McWrite>(&self, w: &mut W) -> McResult<()> {
        let x = self.x as u64;
        let y = self.y as u64;
        let z = self.z as u64;
        let value = ((x & 0x3FFFFFF) << 38) | ((z & 0x3FFFFFF) << 12) | (y & 0xFFF);

        let ivalue = i64::from_ne_bytes(value.to_ne_bytes());
        LongField::new(ivalue).write_field(w).await
    }
}

impl Display for PositionField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl From<(i32, i32, i32)> for PositionField {
    fn from((x, y, z): (i32, i32, i32)) -> Self {
        PositionField { x, y, z }
    }
}
