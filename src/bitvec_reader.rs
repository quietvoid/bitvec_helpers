use anyhow::{bail, ensure, Result};

use bitvec::prelude::*;
use funty::Integral;
use std::fmt;

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
        if let Some(val) = self.bs.get(self.offset) {
            self.offset += 1;

            Ok(*val)
        } else {
            bail!("get: out of bounds");
        }
    }

    #[inline(always)]
    pub fn get_n<I: Integral>(&mut self, n: usize) -> Result<I> {
        let available = self.available();

        ensure!(
            bitvec::mem::bits_of::<I>() <= available,
            "get_n: out of bounds load attempt"
        );
        ensure!(n <= available, "get_n: out of bounds bits");

        let val = self.bs[self.offset..self.offset + n].load_be::<I>();
        self.offset += n;

        Ok(val)
    }

    // bitstring.py implementation: https://github.com/scott-griffiths/bitstring/blob/master/bitstring.py#L1706
    #[inline(always)]
    pub fn get_ue(&mut self) -> Result<u64> {
        let oldpos = self.offset;
        let mut pos = self.offset;

        loop {
            match self.bs.get(pos) {
                Some(val) => {
                    if !val {
                        pos += 1;
                    } else {
                        break;
                    }
                }
                None => bail!("get_ue: out of bounds index: {}", pos),
            }
        }

        let leading_zeroes = pos - oldpos;
        let mut code_num = (1 << leading_zeroes) - 1;

        if leading_zeroes > 0 {
            if pos + leading_zeroes + 1 > self.bs.len() {
                bail!("get_ue: out of bounds attempt");
            }

            code_num += self.bs[pos + 1..pos + leading_zeroes + 1].load_be::<u64>();
            pos += leading_zeroes + 1;
        } else {
            assert_eq!(code_num, 0);
            pos += 1;
        }

        self.offset = pos;

        Ok(code_num)
    }

    // bitstring.py implementation: https://github.com/scott-griffiths/bitstring/blob/master/bitstring.py#L1767
    #[inline(always)]
    pub fn get_se(&mut self) -> Result<i64> {
        let code_num = self.get_ue()?;

        let m = ((code_num + 1) as f64 / 2.0).floor() as u64;

        let val = if code_num % 2 == 0 {
            -(m as i64)
        } else {
            m as i64
        };

        Ok(val)
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
        if self.offset + n > self.available() {
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
    assert!(reader.get_n::<u16>(4).is_err());

    assert!(reader.get_n::<u8>(8).is_ok());
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
