use bitvec::{prelude::Msb0, vec::BitVec, view::BitView};
use funty::Integral;

use crate::signed_to_unsigned;

#[derive(Debug, Default)]
pub struct BitVecWriter {
    bs: BitVec<u8, Msb0>,
    offset: usize,
}

impl BitVecWriter {
    pub fn new() -> Self {
        Self {
            bs: BitVec::new(),
            offset: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bs: BitVec::with_capacity(capacity),
            offset: 0,
        }
    }

    #[inline(always)]
    pub fn write(&mut self, v: bool) {
        self.bs.push(v);
        self.offset += 1;
    }

    #[inline(always)]
    pub fn write_n<T: Integral>(&mut self, v: &T, n: usize)
    where
        T::Bytes: BitView,
    {
        let bytes = v.to_be_bytes();
        let slice = bytes.view_bits::<Msb0>();

        let bits_diff = if n as u32 > T::BITS {
            (n as u32 - T::BITS) as usize
        } else {
            0
        };

        if bits_diff > 0 {
            self.bs.extend(std::iter::repeat(false).take(bits_diff));
            self.bs.extend_from_bitslice(slice);
        } else {
            self.bs.extend_from_bitslice(&slice[slice.len() - n..]);
        }

        self.offset += n;
    }

    #[inline(always)]
    pub fn write_ue(&mut self, v: &u64) {
        if *v == 0 {
            self.bs.push(true);
            self.offset += 1;
        } else {
            let mut tmp = v + 1;
            let mut leading_zeroes: i64 = -1;

            while tmp > 0 {
                tmp >>= 1;
                leading_zeroes += 1;
            }

            let leading_zeroes = leading_zeroes as usize;
            let bits_iter = std::iter::repeat(false)
                .take(leading_zeroes)
                .chain(std::iter::once(true));

            self.bs.extend(bits_iter);
            self.offset += leading_zeroes + 1;

            let remaining = v + 1 - (1 << leading_zeroes);
            self.write_n(&remaining, leading_zeroes);
        }
    }

    #[inline(always)]
    pub fn write_se(&mut self, v: &i64) {
        self.write_ue(&signed_to_unsigned(v));
    }

    #[inline(always)]
    pub fn is_aligned(&self) -> bool {
        self.offset % 8 == 0
    }

    #[inline(always)]
    pub fn written_bits(&self) -> usize {
        self.offset
    }

    pub fn as_slice(&self) -> &[u8] {
        self.bs.as_raw_slice()
    }
}
