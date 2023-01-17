#[cfg(feature = "bitvec")]
mod bitvec_impl;

#[cfg(feature = "bitvec")]
pub use bitvec_impl::{bitslice_reader, bitvec_reader, bitvec_writer};

#[cfg(feature = "bitstream-io")]
mod bitstream_io_impl;

#[cfg(feature = "bitstream-io")]
pub use bitstream_io_impl::{bitstream_io_reader, bitstream_io_writer};

pub(crate) fn signed_to_unsigned(v: &i64) -> u64 {
    let u = if v.is_positive() { (v * 2) - 1 } else { -2 * v };

    u as u64
}
