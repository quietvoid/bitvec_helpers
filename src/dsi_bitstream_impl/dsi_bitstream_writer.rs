use anyhow::Result;

use dsi_bitstream::{
    impls::{BufBitWriter, MemWordWriterVec},
    traits::{BitWrite, WordSeek, BE},
};
use num_traits::{AsPrimitive, Signed, WrappingNeg, WrappingShl};

use crate::signed_to_unsigned;

pub struct DsiBitstreamWriter {
    bw: BufBitWriter<BE, MemWordWriterVec<u8, Vec<u8>>>,
    offset: usize,
}

impl DsiBitstreamWriter {
    pub fn with_capacity(capacity: usize) -> Self {
        let buf = Vec::with_capacity(capacity);

        Self {
            bw: BufBitWriter::new(MemWordWriterVec::new(buf)),
            offset: 0,
        }
    }

    pub fn from_vec(buf: Vec<u8>) -> Result<Self> {
        let offset = buf.len();
        let mut writer = MemWordWriterVec::new(buf);
        writer.set_word_pos(offset as u64)?;

        Ok(Self {
            bw: BufBitWriter::new(writer),
            offset,
        })
    }

    #[inline(always)]
    pub fn write(&mut self, v: bool) -> Result<()> {
        self.bw.write_bits(v as u64, 1)?;
        self.offset += 1;
        Ok(())
    }

    #[inline(always)]
    pub fn write_n<T: AsPrimitive<u64>>(&mut self, v: &T, n: u32) -> Result<()> {
        let n = n as usize;

        self.bw.write_bits(v.as_(), n)?;
        self.offset += n;
        Ok(())
    }
    #[inline(always)]
    pub fn write_signed_n<T: Signed + WrappingNeg + WrappingShl + AsPrimitive<u64>>(
        &mut self,
        v: &T,
        n: u32,
    ) -> Result<()> {
        if v.is_negative() {
            let v = *v - T::one().neg().wrapping_shl(n - 1);
            self.write(true)?;
            self.write_n(&v, n - 1)?;
        } else {
            self.write(false)?;
            self.write_n(v, n - 1)?;
        }

        Ok(())
    }

    #[inline(always)]
    pub fn write_ue(&mut self, v: &u64) -> Result<()> {
        if *v == 0 {
            self.write(true)
        } else {
            let mut tmp = v + 1;
            let mut leading_zeroes: i64 = -1;

            while tmp > 0 {
                tmp >>= 1;
                leading_zeroes += 1;
            }

            for _ in 0..leading_zeroes {
                self.bw.write_bits(0, 1)?;
            }
            self.bw.write_bits(1, 1)?;
            self.offset += leading_zeroes as usize + 1;

            let leading_zeroes = leading_zeroes as u32;
            let remaining = v + 1 - (1 << leading_zeroes);

            self.write_n(&remaining, leading_zeroes)
        }
    }

    #[inline(always)]
    pub fn write_se(&mut self, v: &i64) -> Result<()> {
        self.write_ue(&signed_to_unsigned(v))
    }

    #[inline(always)]
    pub fn is_aligned(&self) -> bool {
        self.offset % 8 == 0
    }

    pub fn byte_align(&mut self) -> Result<()> {
        while !self.is_aligned() {
            self.write(false)?;
        }

        Ok(())
    }

    pub fn into_inner(self) -> Result<Vec<u8>> {
        let buf = self.bw.into_inner().map(|w| w.into_inner())?;
        Ok(buf)
    }
}
