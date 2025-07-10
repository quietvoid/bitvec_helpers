use std::io;

use bitstream_io::{BigEndian, BitWrite, BitWriter, Integer};

use crate::signed_to_unsigned;

pub struct BitstreamIoWriter(BitWriter<Vec<u8>, BigEndian>);

impl BitstreamIoWriter {
    pub fn with_capacity(capacity: usize) -> Self {
        let write = Vec::with_capacity(capacity);

        Self(BitWriter::new(write))
    }

    #[inline(always)]
    pub fn write_bit(&mut self, bit: bool) -> io::Result<()> {
        self.0.write_bit(bit)
    }

    #[inline(always)]
    pub fn write<const BITS: u32, I: Integer>(&mut self, value: I) -> io::Result<()> {
        self.0.write::<BITS, I>(value)
    }

    #[inline(always)]
    pub fn write_const<const BITS: u32, const VALUE: u32>(&mut self) -> io::Result<()> {
        self.0.write_const::<BITS, VALUE>()
    }

    #[inline(always)]
    pub fn write_var<I: Integer>(&mut self, bits: u32, value: I) -> io::Result<()> {
        self.0.write_var(bits, value)
    }

    #[inline(always)]
    pub fn write_ue(&mut self, v: u64) -> io::Result<()> {
        if v == 0 {
            self.write_bit(true)
        } else {
            let mut tmp = v + 1;
            let mut leading_zeroes: i64 = -1;

            while tmp > 0 {
                tmp >>= 1;
                leading_zeroes += 1;
            }

            let leading_zeroes = leading_zeroes as u32;
            self.0.write_unary::<1>(leading_zeroes)?;

            let remaining = v + 1 - (1 << leading_zeroes);
            self.write_var(leading_zeroes, remaining)
        }
    }

    #[inline(always)]
    pub fn write_se(&mut self, v: i64) -> io::Result<()> {
        self.write_ue(signed_to_unsigned(v))
    }

    #[inline(always)]
    pub fn write_bytes(&mut self, buf: &[u8]) -> io::Result<()> {
        self.0.write_bytes(buf)
    }

    #[inline(always)]
    pub fn byte_aligned(&self) -> bool {
        self.0.byte_aligned()
    }

    #[inline(always)]
    pub fn byte_align(&mut self) -> io::Result<()> {
        self.0.byte_align()
    }

    #[inline(always)]
    pub fn pad(&mut self, bits: u32) -> io::Result<()> {
        self.0.pad(bits)
    }

    /// None if the writer is not byte aligned
    pub fn as_slice(&mut self) -> Option<&[u8]> {
        self.0.writer().map(|e| e.as_slice())
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0.into_writer()
    }
}
