use anyhow::{bail, Result};

use bitvec::{prelude::Msb0, slice::BitSlice, view::BitView};
use funty::Integral;
use std::fmt;

use super::reads::BitSliceReadExt;

#[derive(Default)]
pub struct BitSliceReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> BitSliceReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn bits(&self) -> &BitSlice<u8, Msb0> {
        self.bytes.view_bits::<Msb0>()
    }

    #[inline(always)]
    pub fn get(&mut self) -> Result<bool> {
        BitSliceReadExt::get(self.bytes.view_bits::<Msb0>(), &mut self.offset)
    }

    #[inline(always)]
    pub fn get_n<I: Integral>(&mut self, n: usize) -> Result<I> {
        BitSliceReadExt::get_n(self.bytes.view_bits::<Msb0>(), n, &mut self.offset)
    }

    #[inline(always)]
    pub fn get_ue(&mut self) -> Result<u64> {
        BitSliceReadExt::get_ue(self.bytes.view_bits::<Msb0>(), &mut self.offset)
    }

    #[inline(always)]
    pub fn get_se(&mut self) -> Result<i64> {
        BitSliceReadExt::get_se(self.bytes.view_bits::<Msb0>(), &mut self.offset)
    }

    #[inline(always)]
    pub fn is_aligned(&self) -> bool {
        self.offset % 8 == 0
    }

    #[inline(always)]
    pub fn available(&self) -> usize {
        self.bits().len() - self.offset
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
        &self.bits()[self.offset..]
    }

    #[inline(always)]
    pub fn position(&self) -> usize {
        self.offset
    }
}

impl fmt::Debug for BitSliceReader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BitSliceReader: {{offset: {}, len: {}}}",
            self.offset,
            self.bytes.len()
        )
    }
}

#[test]
fn get_n_validations() {
    let mut reader = BitSliceReader::new(&[1]);
    assert!(reader.get_n::<u8>(9).is_err());
    assert!(reader.get_n::<u16>(4).is_ok());

    assert!(reader.get_n::<u8>(8).is_err());
    assert!(reader.get_n::<u8>(4).is_ok());
    assert!(reader.get().is_err());
}

#[test]
fn skip_n_validations() {
    let mut reader = BitSliceReader::new(&[1]);
    assert!(reader.skip_n(9).is_err());

    assert!(reader.skip_n(7).is_ok());
    assert!(reader.get().is_ok());
    assert!(reader.get().is_err());
}
