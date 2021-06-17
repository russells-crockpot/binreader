#![allow(unused_imports, dead_code, unused_variables, unused_mut)]
#![allow(clippy::needless_range_loop)]
use binreader_macros::{impl_at_methods, impl_next_methods, make_number_methods};
use bytes::Bytes;
use std::{borrow::Borrow, io, path::Path};

// Needed for some macros to work in this package.
use crate as binreader;

pub mod iter;
pub mod util;

#[macro_use]
extern crate binreader_internal_macros;

#[cfg(feature = "memmap")]
mod mmap;

#[cfg(test)]
mod testing;

mod consuming;
mod random_access;
mod slice;

pub use consuming::ConsumingBinReader;
#[cfg(feature = "memmap")]
pub use mmap::MmapBinReader;
pub use random_access::RandomAccessBinReader;
pub use slice::{SliceAsRefReader, SliceRefReader};

#[derive(Debug, Clone, Copy)]
pub enum Endidness {
    Big,
    Little,
    Unknown,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An offset of 0x{0:x} is too small.")]
    OffsetTooSmall(usize),
    #[error("An offset of 0x{0:x} is too large.")]
    OffsetTooLarge(usize),
    #[error("Requested {0} bytes, but only {1} bytes left.")]
    /// NotEnoughData(bytes requests, bytes remaining)
    NotEnoughData(usize, usize),
    #[error("Attempted to call a method that requires knowing the endidness.")]
    UnknownEndidness,
    #[error("No more data left.")]
    NoMoreData,
    #[error("{0}")]
    IoError(io::Error),
    #[error("{0}")]
    Other(String),
    //#[error("Received invalid data.")]
    //InvalidData,
}

impl From<Error> for io::Error {
    fn from(e: Error) -> Self {
        io::Error::new(io::ErrorKind::Other, e)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

pub type Result<V> = std::result::Result<V, Error>;

pub trait BinReader<'r>
where
    Self: Sized + AsRef<[u8]>,
{
    fn from_slice_with_offset(
        slice: &'r [u8],
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self>;

    fn from_slice(slice: &'r [u8], endidness: Endidness) -> Result<Self> {
        Self::from_slice_with_offset(slice, 0, endidness)
    }

    fn size(&self) -> usize;

    fn initial_offset(&self) -> usize;

    fn current_offset(&self) -> usize;

    fn endidness(&self) -> Endidness;

    fn advance_to(&mut self, offset: usize) -> Result<()>;

    fn advance_by(&mut self, num_bytes: usize) -> Result<()>;

    fn next_u8(&mut self) -> Result<u8> {
        let byte = self.current_byte()?;
        self.advance_by(1)?;
        Ok(byte)
    }

    fn next_n_bytes(&mut self, num_bytes: usize) -> Result<Bytes> {
        self.validate_offset(self.current_offset(), num_bytes)?;
        let start = self.current_offset() + self.initial_offset();
        let data = Bytes::copy_from_slice(&self.as_ref()[start..start + num_bytes]);
        self.advance_by(num_bytes)?;
        Ok(data)
    }

    #[inline]
    fn lower_offset_limit(&self) -> usize {
        self.initial_offset()
    }

    #[inline]
    fn upper_offset_limit(&self) -> usize {
        self.size() + self.initial_offset()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.upper_offset_limit() - self.current_offset()
    }

    fn validate_offset(&self, offset: usize, size: usize) -> Result<()> {
        if size > 0 && self.is_empty() {
            Err(Error::NoMoreData)
        } else if offset < self.lower_offset_limit() {
            Err(Error::OffsetTooSmall(offset))
        } else if offset > self.upper_offset_limit() {
            Err(Error::OffsetTooLarge(offset))
        } else if offset > self.upper_offset_limit() - size {
            Err(Error::NotEnoughData(size, self.remaining()))
        } else {
            Ok(())
        }
    }

    fn relative_offset(&self, abs_offset: usize) -> Result<usize> {
        self.validate_offset(abs_offset, 0)?;
        Ok(abs_offset - self.current_offset())
    }

    /// Returns `true` if the next bytes are the same as the ones provided.
    fn next_bytes_are(&self, prefix: &[u8]) -> Result<bool> {
        self.validate_offset(self.current_offset(), prefix.len())?;
        let mut buf = Vec::with_capacity(prefix.len());
        (0..buf.len()).for_each(|_| buf.push(0));
        self.bytes_at(self.current_offset(), &mut buf)?;
        Ok(prefix.iter().zip(buf.into_iter()).all(|(v1, v2)| *v1 == v2))
    }

    fn bytes_at(&self, offset: usize, buf: &mut [u8]) -> Result<()> {
        self.validate_offset(offset, buf.len())?;
        for i in 0..buf.len() {
            buf[i] = self.u8_at(offset + i)?;
        }
        Ok(())
    }

    fn subseq(&self, offset: usize, num_bytes: usize) -> Result<&[u8]> {
        self.range(offset, offset + num_bytes)
    }

    fn range(&self, start: usize, end: usize) -> Result<&[u8]> {
        Ok(&self.as_ref()[start..end])
    }

    fn next_bytes(&mut self, buf: &mut [u8]) -> Result<()> {
        for i in 0..buf.len() {
            buf[i] = self.next_u8()?;
        }
        Ok(())
    }

    fn current_byte(&self) -> Result<u8> {
        self.u8_at(self.current_offset())
    }

    fn u8_at(&self, offset: usize) -> Result<u8> {
        self.validate_offset(offset, 0)?;
        Ok(self.as_ref()[offset - self.initial_offset()])
    }

    make_number_methods! {
        fn numname_numend_at(&self, offset: usize) -> Result<_numname_> {
            let mut buf = [0; _numwidth_];
            self.bytes_at(offset, &mut buf)?;
            Ok(_numname_::from_numend_bytes(buf))
        }
    }

    fn u16_at(&self, offset: usize) -> Result<u16> {
        match self.endidness() {
            Endidness::Big => self.u16_be_at(offset),
            Endidness::Little => self.u16_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    fn u32_at(&self, offset: usize) -> Result<u32> {
        match self.endidness() {
            Endidness::Big => self.u32_be_at(offset),
            Endidness::Little => self.u32_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    fn u64_at(&self, offset: usize) -> Result<u64> {
        match self.endidness() {
            Endidness::Big => self.u64_be_at(offset),
            Endidness::Little => self.u64_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    fn u128_at(&self, offset: usize) -> Result<u128> {
        match self.endidness() {
            Endidness::Big => self.u128_be_at(offset),
            Endidness::Little => self.u128_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    fn i16_at(&self, offset: usize) -> Result<i16> {
        match self.endidness() {
            Endidness::Big => self.i16_be_at(offset),
            Endidness::Little => self.i16_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    fn i32_at(&self, offset: usize) -> Result<i32> {
        match self.endidness() {
            Endidness::Big => self.i32_be_at(offset),
            Endidness::Little => self.i32_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    fn i64_at(&self, offset: usize) -> Result<i64> {
        match self.endidness() {
            Endidness::Big => self.i64_be_at(offset),
            Endidness::Little => self.i64_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    fn i128_at(&self, offset: usize) -> Result<i128> {
        match self.endidness() {
            Endidness::Big => self.i128_be_at(offset),
            Endidness::Little => self.i128_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    fn next_i8(&mut self) -> Result<i8> {
        let mut buf = [0; 1];
        self.next_bytes(&mut buf)?;
        Ok(i8::from_be_bytes(buf))
    }

    make_number_methods! {
        fn next_numname_numend(&mut self) -> Result<_numname_> {
            let mut buf = [0; _numwidth_];
            self.next_bytes(&mut buf)?;
            Ok(_numname_::from_numend_bytes(buf))
        }
    }

    fn next_u16(&mut self) -> Result<u16> {
        match self.endidness() {
            Endidness::Big => self.next_u16_be(),
            Endidness::Little => self.next_u16_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    fn next_u32(&mut self) -> Result<u32> {
        match self.endidness() {
            Endidness::Big => self.next_u32_be(),
            Endidness::Little => self.next_u32_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    fn next_u64(&mut self) -> Result<u64> {
        match self.endidness() {
            Endidness::Big => self.next_u64_be(),
            Endidness::Little => self.next_u64_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    fn next_u128(&mut self) -> Result<u128> {
        match self.endidness() {
            Endidness::Big => self.next_u128_be(),
            Endidness::Little => self.next_u128_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    fn next_i16(&mut self) -> Result<i16> {
        match self.endidness() {
            Endidness::Big => self.next_i16_be(),
            Endidness::Little => self.next_i16_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    fn next_i32(&mut self) -> Result<i32> {
        match self.endidness() {
            Endidness::Big => self.next_i32_be(),
            Endidness::Little => self.next_i32_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    fn next_i64(&mut self) -> Result<i64> {
        match self.endidness() {
            Endidness::Big => self.next_i64_be(),
            Endidness::Little => self.next_i64_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    fn next_i128(&mut self) -> Result<i128> {
        match self.endidness() {
            Endidness::Big => self.next_i128_be(),
            Endidness::Little => self.next_i128_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }
}

pub trait OwnableBinReader<'r>: BinReader<'r> {
    fn from_file_with_offset<P: AsRef<Path>>(
        path: P,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self>;

    fn from_file<P: AsRef<Path>>(path: P, endidness: Endidness) -> Result<Self> {
        Self::from_file_with_offset(path, 0, endidness)
    }

    fn from_bytes_with_offset(
        bytes: Bytes,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self>;

    fn from_bytes(bytes: Bytes, endidness: Endidness) -> Result<Self> {
        Self::from_bytes_with_offset(bytes, 0, endidness)
    }
}
