use anyhow::{bail, Context, Result};
use dsi_bitstream::{
    codes::ExpGolombRead,
    impls::{BufBitReader, MemWordReader},
    traits::{BitRead, BitSeek, BE},
};
use num_traits::{FromPrimitive, Zero};

pub struct DsiBitstreamReader<BR: BitRead<BE>> {
    bs: BR,
    len: u64,
}

/// Convenience type for &[u8] inner buffer
pub type DsiBsSliceReader<'a> =
    DsiBitstreamReader<BufBitReader<BE, MemWordReader<u32, &'a [u32], false>>>;

impl<BR> DsiBitstreamReader<BR>
where
    BR: BitRead<BE> + BitSeek + ExpGolombRead<BE>,
{
    pub fn new(bs: BR, len_bytes: u64) -> Self {
        Self {
            bs,
            len: len_bytes * 8,
        }
    }

    #[inline(always)]
    pub fn get(&mut self) -> Result<bool, <BR as BitRead<BE>>::Error> {
        self.bs.read_bits(1).map(|v| v == 1)
    }

    #[inline(always)]
    pub fn get_n<T: FromPrimitive>(&mut self, n: u32) -> Result<T> {
        if self.available().is_ok_and(|avail| n as u64 > avail) {
            bail!("get_n: out of bounds bits");
        }

        let res = self.bs.read_bits(n as usize).map(T::from_u64)?;
        res.context("Value does not fit in type")
    }

    #[inline(always)]
    pub fn get_ue(&mut self) -> Result<u64> {
        let mut leading_zeroes = 0;
        while self.bs.read_bits(1)?.is_zero() {
            leading_zeroes += 1;
        }

        let res = if leading_zeroes > 0 {
            self.bs
                .read_bits(leading_zeroes)
                .map(|v| v + (1 << leading_zeroes) - 1)?
        } else {
            0
        };

        Ok(res)
    }

    #[inline(always)]
    pub fn get_se(&mut self) -> Result<i64> {
        self.get_ue().map(|code_num| {
            let m = ((code_num + 1) as f64 / 2.0).floor() as u64;

            if code_num % 2 == 0 {
                -(m as i64)
            } else {
                m as i64
            }
        })
    }

    #[inline(always)]
    pub fn is_aligned(&mut self) -> bool {
        self.bs.bit_pos().is_ok_and(|pos| pos % 8 == 0)
    }

    #[inline(always)]
    pub fn available(&mut self) -> Result<u64> {
        Ok(self.bs.bit_pos().map(|pos| self.len - pos)?)
    }

    pub fn skip_n(&mut self, n: usize) -> Result<()> {
        if self.available().is_ok_and(|avail| n as u64 > avail) {
            bail!("skip_n: out of bounds bits");
        }

        Ok(self.bs.skip_bits(n)?)
    }

    #[inline(always)]
    pub fn position(&mut self) -> Result<u64> {
        Ok(self.bs.bit_pos()?)
    }

    pub fn replace_inner(&mut self, read: BR, len_bytes: u64) {
        self.len = len_bytes * 8;
        self.bs = read;
    }
}

impl<'a> DsiBsSliceReader<'a> {
    pub fn from_slice(buf: &'a [u8]) -> Self {
        let len = buf.len() as u64;

        let data = unsafe { std::mem::transmute::<_, &[u32]>(buf) };
        let br = BufBitReader::new(MemWordReader::new_strict(data));
        Self::new(br, len)
    }
}

#[test]
fn get_n_validations() {
    let mut reader = DsiBsSliceReader::from_slice(&[1]);
    assert!(reader.get_n::<u8>(9).is_err());
    assert!(reader.get_n::<u16>(4).is_ok());

    assert!(reader.get_n::<u8>(8).is_err());
    assert!(reader.get_n::<u8>(4).is_ok());
    assert!(reader.get().is_err());
}

#[test]
fn skip_n_validations() {
    let mut reader = DsiBsSliceReader::from_slice(&[1]);
    assert!(reader.skip_n(9).is_err());

    assert!(reader.skip_n(7).is_ok());
    assert!(reader.get().is_ok());
    assert!(reader.get().is_err());
}
