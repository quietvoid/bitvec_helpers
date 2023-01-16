use anyhow::{anyhow, bail, ensure, Result};
use bitvec::{field::BitField, prelude::Msb0, slice::BitSlice};
use funty::Integral;

pub(crate) struct BitSliceReadExt {}

impl BitSliceReadExt {
    #[inline(always)]
    pub fn get(slice: &BitSlice<u8, Msb0>, offset: &mut usize) -> Result<bool> {
        let val = {
            *slice
                .get(*offset)
                .ok_or_else(|| anyhow!("get: out of bounds"))?
        };

        *offset += 1;

        Ok(val)
    }

    #[inline(always)]
    pub fn get_n<I: Integral>(
        slice: &BitSlice<u8, Msb0>,
        n: usize,
        offset: &mut usize,
    ) -> Result<I> {
        let pos = *offset;
        let available = slice.len() - pos;

        ensure!(n <= available, "get_n: out of bounds bits");

        let val = slice[pos..pos + n].load_be::<I>();
        *offset += n;

        Ok(val)
    }

    // bitstring.py implementation: https://github.com/scott-griffiths/bitstring/blob/master/bitstring.py#L1706
    #[inline(always)]
    pub fn get_ue(slice: &BitSlice<u8, Msb0>, offset: &mut usize) -> Result<u64> {
        let oldpos = *offset;
        let mut pos = oldpos;

        loop {
            match slice.get(pos) {
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
        let endpos = pos + leading_zeroes + 1;

        let code_num = if leading_zeroes > 0 {
            if endpos > slice.len() {
                bail!("get_ue: out of bounds attempt");
            }

            let code_num = (1 << leading_zeroes) - 1;

            code_num + slice[pos + 1..endpos].load_be::<u64>()
        } else {
            0
        };

        *offset = endpos;

        Ok(code_num)
    }

    // bitstring.py implementation: https://github.com/scott-griffiths/bitstring/blob/master/bitstring.py#L1767
    #[inline(always)]
    pub fn get_se(slice: &BitSlice<u8, Msb0>, offset: &mut usize) -> Result<i64> {
        let code_num = Self::get_ue(slice, offset)?;

        let m = ((code_num + 1) as f64 / 2.0).floor() as u64;

        let val = if code_num % 2 == 0 {
            -(m as i64)
        } else {
            m as i64
        };

        Ok(val)
    }
}
