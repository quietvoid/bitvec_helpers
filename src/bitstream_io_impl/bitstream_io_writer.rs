use bitstream_io::{BigEndian, BitWrite, BitWriter, Numeric, SignedNumeric};
use std::io;

use crate::signed_to_unsigned;

pub struct BitstreamIoWriter(BitWriter<Vec<u8>, BigEndian>);

impl BitstreamIoWriter {
    pub fn with_capacity(capacity: usize) -> Self {
        let write = Vec::with_capacity(capacity);

        Self(BitWriter::new(write))
    }

    #[inline(always)]
    pub fn write(&mut self, v: bool) -> io::Result<()> {
        self.0.write_bit(v)
    }

    #[inline(always)]
    pub fn write_n<T: Numeric>(&mut self, v: &T, n: u32) -> io::Result<()> {
        self.0.write(n, *v)
    }

    #[inline(always)]
    pub fn write_signed_n<T: SignedNumeric>(&mut self, v: &T, n: u32) -> io::Result<()> {
        self.0.write_signed(n, *v)
    }

    #[inline(always)]
    pub fn write_ue(&mut self, v: &u64) -> io::Result<()> {
        if *v == 0 {
            self.write(true)
        } else {
            let mut tmp = v + 1;
            let mut leading_zeroes: i64 = -1;

            while tmp > 0 {
                tmp >>= 1;
                leading_zeroes += 1;
            }

            let leading_zeroes = leading_zeroes as u32;
            self.0.write_unary1(leading_zeroes)?;

            let remaining = v + 1 - (1 << leading_zeroes);
            self.write_n(&remaining, leading_zeroes)
        }
    }

    #[inline(always)]
    pub fn write_se(&mut self, v: &i64) -> io::Result<()> {
        self.write_ue(&signed_to_unsigned(v))
    }

    #[inline(always)]
    pub fn is_aligned(&self) -> bool {
        self.0.byte_aligned()
    }

    pub fn byte_align(&mut self) -> std::io::Result<()> {
        self.0.byte_align()
    }

    /// None if the writer is not byte aligned
    pub fn as_slice(&mut self) -> Option<&[u8]> {
        self.0.writer().map(|e| e.as_slice())
    }
}
