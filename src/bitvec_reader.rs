use anyhow::{bail, Result};

use bitvec::{prelude::Msb0, slice::BitSlice, vec::BitVec};
use funty::Integral;
use std::fmt;

use super::reads::BitSliceReadExt;

#[derive(Default)]
pub struct BitVecReader {
    bs: BitVec<u8, Msb0>,
    offset: usize,
}

impl BitVecReader {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            bs: BitVec::from_vec(data),
            offset: 0,
        }
    }

    #[inline(always)]
    pub fn get(&mut self) -> Result<bool> {
        BitSliceReadExt::get(self.bs.as_bitslice(), &mut self.offset)
    }

    #[inline(always)]
    pub fn get_n<I: Integral>(&mut self, n: usize) -> Result<I> {
        BitSliceReadExt::get_n(self.bs.as_bitslice(), n, &mut self.offset)
    }

    #[inline(always)]
    pub fn get_ue(&mut self) -> Result<u64> {
        BitSliceReadExt::get_ue(self.bs.as_bitslice(), &mut self.offset)
    }

    #[inline(always)]
    pub fn get_se(&mut self) -> Result<i64> {
        BitSliceReadExt::get_se(self.bs.as_bitslice(), &mut self.offset)
    }

    #[inline(always)]
    pub fn is_aligned(&self) -> bool {
        self.offset % 8 == 0
    }

    #[inline(always)]
    pub fn available(&self) -> usize {
        self.bs.len() - self.offset
    }

    #[inline(always)]
    pub fn skip_n(&mut self, n: usize) -> Result<()> {
        if n > self.available() {
            bail!("Cannot skip more bits than available");
        }

        self.offset += n;
        Ok(())
    }

    pub fn available_slice(&self) -> &BitSlice<u8, Msb0> {
        &self.bs[self.offset..]
    }

    #[inline(always)]
    pub fn position(&self) -> usize {
        self.offset
    }
}

impl fmt::Debug for BitVecReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BitVecReader: {{offset: {}, len: {}}}",
            self.offset,
            self.bs.len()
        )
    }
}

#[test]
fn get_n_validations() {
    let mut reader = BitVecReader::new(vec![1]);
    assert!(reader.get_n::<u8>(9).is_err());
    assert!(reader.get_n::<u16>(4).is_ok());

    assert!(reader.get_n::<u8>(8).is_err());
    assert!(reader.get_n::<u8>(4).is_ok());
    assert!(reader.get().is_err());
}

#[test]
fn skip_n_validations() {
    let mut reader = BitVecReader::new(vec![1]);
    assert!(reader.skip_n(9).is_err());

    assert!(reader.skip_n(7).is_ok());
    assert!(reader.get().is_ok());
    assert!(reader.get().is_err());
}
