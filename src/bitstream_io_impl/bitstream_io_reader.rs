use std::io;

use bitstream_io::{BigEndian, BitRead, BitReader, Numeric};

pub struct BitstreamIoReader<R: io::Read + io::Seek> {
    bs: BitReader<R, BigEndian>,
    len: u64,
}

/// Convenience type for Vec<u8> inner buffer
pub type BsIoVecReader = BitstreamIoReader<io::Cursor<Vec<u8>>>;

/// Convenience type for &[u8] inner buffer
pub type BsIoSliceReader<'a> = BitstreamIoReader<io::Cursor<&'a [u8]>>;

impl<R> BitstreamIoReader<R>
where
    R: io::Read + io::Seek,
{
    pub fn new(read: R, len_bytes: u64) -> Self {
        Self {
            bs: BitReader::new(read),
            len: len_bytes * 8,
        }
    }

    #[inline(always)]
    pub fn get(&mut self) -> io::Result<bool> {
        self.bs.read_bit()
    }

    #[inline(always)]
    pub fn get_n<T: Numeric>(&mut self, n: u32) -> io::Result<T> {
        self.available().and_then(|avail| {
            if n as u64 > avail {
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "get_n: out of bounds bits",
                ))
            } else {
                self.bs.read(n)
            }
        })
    }

    #[inline(always)]
    pub fn get_ue(&mut self) -> io::Result<u64> {
        self.bs.read_unary1().and_then(|leading_zeroes| {
            if leading_zeroes > 0 {
                self.bs
                    .read::<u64>(leading_zeroes)
                    .map(|v| v + (1 << leading_zeroes) - 1)
            } else {
                Ok(0)
            }
        })
    }

    #[inline(always)]
    pub fn get_se(&mut self) -> io::Result<i64> {
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
    pub fn is_aligned(&self) -> bool {
        self.bs.byte_aligned()
    }

    #[inline(always)]
    pub fn available(&mut self) -> io::Result<u64> {
        self.bs.position_in_bits().map(|pos| self.len - pos)
    }

    #[inline(always)]
    pub fn skip_n(&mut self, n: u32) -> io::Result<()> {
        self.available().and_then(|avail| {
            if n as u64 > avail {
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "skip_n: out of bounds bits",
                ))
            } else {
                self.bs.skip(n)
            }
        })
    }

    #[inline(always)]
    pub fn position(&mut self) -> io::Result<u64> {
        self.bs.position_in_bits()
    }

    pub fn replace_inner(&mut self, read: R, len_bytes: u64) {
        self.len = len_bytes * 8;
        self.bs = BitReader::new(read);
    }
}

impl BsIoVecReader {
    pub fn from_vec(buf: Vec<u8>) -> Self {
        let len = buf.len() as u64;
        let read = io::Cursor::new(buf);

        Self::new(read, len)
    }

    pub fn replace_vec(&mut self, buf: Vec<u8>) {
        let len = buf.len() as u64;
        self.replace_inner(io::Cursor::new(buf), len);
    }
}

impl<'a> BsIoSliceReader<'a> {
    pub fn from_slice(buf: &'a [u8]) -> Self {
        let len = buf.len() as u64;
        let read = io::Cursor::new(buf);

        Self::new(read, len)
    }

    pub fn replace_slice(&mut self, buf: &'a [u8]) {
        let len = buf.len() as u64;
        self.replace_inner(io::Cursor::new(buf), len);
    }
}

impl Default for BsIoVecReader {
    fn default() -> Self {
        Self::from_vec(Vec::new())
    }
}

impl<'a> Default for BsIoSliceReader<'a> {
    fn default() -> Self {
        Self::from_slice(&[])
    }
}

#[test]
fn get_n_validations() {
    let mut reader = BsIoSliceReader::from_slice(&[1]);
    assert!(reader.get_n::<u8>(9).is_err());
    assert!(reader.get_n::<u16>(4).is_ok());

    assert!(reader.get_n::<u8>(8).is_err());
    assert!(reader.get_n::<u8>(4).is_ok());
    assert!(reader.get().is_err());
}

#[test]
fn skip_n_validations() {
    let mut reader = BsIoSliceReader::from_slice(&[1]);
    assert!(reader.skip_n(9).is_err());

    assert!(reader.skip_n(7).is_ok());
    assert!(reader.get().is_ok());
    assert!(reader.get().is_err());
}
