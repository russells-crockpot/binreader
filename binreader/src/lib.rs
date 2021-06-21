//! Helpers to read binary data files in rust
//!
//! # Overview
//!
//! The binreader crate is designed to make reading binary data easier. It is not
//! meant to replace other wonderful crates like
//! [bytes](https://github.com/tokio-rs/bytes),
//! [nom](https://github.com/Geal/nom) or [binread](https://github.com/jam1garner/binread),
//! but instead is meant to work with them as a single, common interface between.
//!
//! This is primarily done via the [`BinReader`] trait, as well as a variety of different
//! implementations of it useful for a variety of purposes.
//!
//! # Feature Flags
//!
//! As of right now, BinReader only has two feature flags:
//!
//! - `nom-support` which allows [nom](https://github.com/Geal/nom) to parse from
//!   BinReaders.
//! - `memmap` which supports platform-independent memory mapped files (via the
//!   [memmap2](https://github.com/RazrFalcon/memmap2-rs) crate).

#![allow(clippy::needless_range_loop)]
use binreader_macros::make_number_methods;
use bytes::Bytes;
use std::{borrow::Borrow, io, path::Path};

// Needed for some macros to work in this package.
#[allow(unused_imports)]
use crate as binreader;

//pub mod iter;
pub mod util;

#[macro_use]
extern crate binreader_internal_macros;

mod readers;
pub use readers::*;

#[cfg(test)]
mod testing;

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

/// The primary trait of this crate; a [`BinReader`] is designed to be a common interface between
/// your program and binary data.
///
/// While not required, most [`BinReader`]s should implement the [`std::io::Read`],
/// [`std::io::Seek`], [`std::io::BufRead`], and ``std::borrow::Borrow<&[u8]>`` traits.
///
/// Additionally, there's the [`OwnableBinReader`] subtrait which owns the data contained within it.
///
/// # Offsets
///
/// Instead of indexes, [`BinReader`]s use offsets. Now, in most cases these are probably going to
/// be the same. However, you can specify an initial offset that will essentially change the index
/// of zero to whatever the initial_offset is.
/// For example:
///
/// ```ignore
/// let test_data = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
/// let reader = RandomAccessBinReader::from_slice_with_offset(&test_data, 100, Endidness::Big);
/// assert_eq!(reader.u8_at(100).unwrap(), 0);
/// ```
///
/// ## Validation
///
/// One thing you may have noticed is that we had to unwrap the value. Most of a [`BinReader`]'s
/// methods first check to make sure the provided offset is valid. For example:
///
/// ```ignore
/// assert!(matches!(reader.u8_at(99), Err(Error::OffsetTooSmall(99))));
/// ```
///
/// ## Limits
///
/// The lower and upper limits of valid offsets can be retrieved via the
/// [`BinReader::lower_offset_limit`] and [`BinReader::upper_offset_limit`] methods respectively.
///
/// An important note: the [BinReader::size] method returns how much *data* is in the reader, not
/// what the highest valid offset is.
pub trait BinReader<'r>
where
    Self: Sized + AsRef<[u8]> + Borrow<[u8]> + io::Read + io::BufRead,
{
    /// Generates a new [`BinReader`] using the provided slice, initial offset, and endidness. While
    /// the exact implementation of this varies from implementation to implementation,
    /// [`OwnableBinReader`]s will, more than likely, copy the data in the slice.
    fn from_slice_with_offset(
        slice: &'r [u8],
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self>;

    /// Functions the same as the [`BinReader::from_slice_with_offset`], except the initial offset
    /// is always `0`.
    fn from_slice(slice: &'r [u8], endidness: Endidness) -> Result<Self> {
        Self::from_slice_with_offset(slice, 0, endidness)
    }

    /// The amount of data in the reader. If the reader's size changes (which none of the
    /// implementations currently do), then this should return how much data was *initially* in the
    /// reader.
    fn size(&self) -> usize;

    /// The initial offset of the [`BinReader`]. For more information, see the **Offsets** section
    /// of the [`BinReader`] documentation.
    fn initial_offset(&self) -> usize;

    /// The current offset of the reader's cursor.
    fn current_offset(&self) -> usize;

    /// The endidness of the reader.
    fn endidness(&self) -> Endidness;

    /// Changes the default endidness.
    fn change_endidness(&mut self, endidness: Endidness);

    /// Sets the reader's [`BinReader::current_offset`].
    fn advance_to(&self, offset: usize) -> Result<()>;

    /// Alters the [`BinReader::current_offset`] by the given amount.
    fn advance_by(&self, num_bytes: isize) -> Result<()>;

    /// Returns a [`Bytes`] object of the requested size containing the next n bytes (where n is
    /// the `num_bytes` parameter) and then advances the cursor by that much.
    fn next_n_bytes(&self, num_bytes: usize) -> Result<&[u8]> {
        self.validate_offset(self.current_offset(), num_bytes)?;
        let start = self.current_offset() - self.initial_offset();
        let data = &self.as_ref()[start..start + num_bytes];
        self.advance_by(num_bytes as isize)?;
        Ok(data)
    }

    #[inline]
    /// Gets a pointer to a slice of the byte at the [`BinReader::current_offset`], as well as all
    /// all bytes afterwards. This does not alter the [`BinReader::current_offset`].
    fn get_remaining(&self) -> Result<&[u8]> {
        self.range(self.current_offset(), self.upper_offset_limit())
    }

    #[inline]
    /// The lowest valid offset that can be requested. By default, this is the same as
    /// [`BinReader::initial_offset`].
    fn lower_offset_limit(&self) -> usize {
        self.initial_offset()
    }

    #[inline]
    /// The highest valid offset that can be requested. By default, this is the reader's
    /// [`BinReader::size`] plus its [`BinReader::initial_offset`].
    fn upper_offset_limit(&self) -> usize {
        self.size() + self.initial_offset()
    }

    #[inline]
    /// Checks whether or not there is any data left, based off of the
    /// [`BinReader::current_offset`].
    fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    #[inline]
    /// The amount of data left, based off of the [`BinReader::current_offset`].
    fn remaining(&self) -> usize {
        self.upper_offset_limit() - self.current_offset()
    }

    /// A helper method that validates an offset (mostly used by reader implementations).
    ///
    /// If the offset is valid, then `Ok(())` will be returned. Otherwise, the appropriate
    /// [`Error`] is returned (wrapped in `Err`, of course).
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

    /// Takes an absolute offset and converts it to a relative offset, based off of the
    /// [`BinReader::current_offset`].
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

    /// Fills the provided buffer with bytes, starting at the provided offset. This does not alter
    /// the [`BinReader::current_offset`].
    fn bytes_at(&self, offset: usize, buf: &mut [u8]) -> Result<()> {
        self.validate_offset(offset, buf.len())?;
        for i in 0..buf.len() {
            buf[i] = self.u8_at(offset + i)?;
        }
        Ok(())
    }

    /// Returns a subsequence (i.e. a `&[u8]`) of data of the requested size beginning at the
    /// provided offset.
    fn subseq(&self, offset: usize, num_bytes: usize) -> Result<&[u8]> {
        self.validate_offset(offset, num_bytes)?;
        self.range(offset, offset + num_bytes)
    }

    /// Returns a slice of the data between the provided starting and ending offsets.
    fn range(&self, start: usize, end: usize) -> Result<&[u8]> {
        self.validate_offset(start, end - start)?;
        Ok(&self.as_ref()[start..end])
    }

    /// Fills the provided buffer with the next n bytes, where n is the length of the buffer. This
    /// then advances the [`BinReader::current_offset`] by n.
    fn next_bytes(&self, buf: &mut [u8]) -> Result<()> {
        for i in 0..buf.len() {
            buf[i] = self.next_u8()?;
        }
        Ok(())
    }

    /// Gets the [`u8`] at the [`BinReader::current_offset`] without altering the
    /// [`BinReader::current_offset`].
    fn current_u8(&self) -> Result<u8> {
        self.u8_at(self.current_offset())
    }

    //TODO current, non-endian implementations.
    make_number_methods! {
        /// Gets the numendlong endian `numname` at the [`BinReader::current_offset`] without
        /// altering the [`BinReader::current_offset`].
        fn current_numname_numend(&self) -> Result<_numname_> {
            let mut buf = [0; _numwidth_];
            self.bytes_at(self.current_offset(), &mut buf)?;
            Ok(_numname_::from_numend_bytes(buf))
        }
    }

    /// Gets the `u8` at the provided offset without altering the [`BinReader::current_offset`].
    fn u8_at(&self, offset: usize) -> Result<u8> {
        self.validate_offset(offset, 0)?;
        Ok(self.as_ref()[offset - self.initial_offset()])
    }

    make_number_methods! {
        /// Gets the numendlong endian `numname` at the provided offset without altering the
        /// [`BinReader::current_offset`].
        fn numname_numend_at(&self, offset: usize) -> Result<_numname_> {
            let mut buf = [0; _numwidth_];
            self.bytes_at(offset, &mut buf)?;
            Ok(_numname_::from_numend_bytes(buf))
        }
    }

    /// Gets the `u16` using the default endidness at the provided offset without altering the
    /// [`BinReader::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    fn u16_at(&self, offset: usize) -> Result<u16> {
        match self.endidness() {
            Endidness::Big => self.u16_be_at(offset),
            Endidness::Little => self.u16_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `u32` using the default endidness at the provided offset without altering the
    /// [`BinReader::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    fn u32_at(&self, offset: usize) -> Result<u32> {
        match self.endidness() {
            Endidness::Big => self.u32_be_at(offset),
            Endidness::Little => self.u32_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `u64` using the default endidness at the provided offset without altering the
    /// [`BinReader::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    fn u64_at(&self, offset: usize) -> Result<u64> {
        match self.endidness() {
            Endidness::Big => self.u64_be_at(offset),
            Endidness::Little => self.u64_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `u128` using the default endidness at the provided offset without altering the
    /// [`BinReader::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    fn u128_at(&self, offset: usize) -> Result<u128> {
        match self.endidness() {
            Endidness::Big => self.u128_be_at(offset),
            Endidness::Little => self.u128_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i16` using the default endidness at the provided offset without altering the
    /// [`BinReader::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    fn i16_at(&self, offset: usize) -> Result<i16> {
        match self.endidness() {
            Endidness::Big => self.i16_be_at(offset),
            Endidness::Little => self.i16_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i32` using the default endidness at the provided offset without altering the
    /// [`BinReader::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    fn i32_at(&self, offset: usize) -> Result<i32> {
        match self.endidness() {
            Endidness::Big => self.i32_be_at(offset),
            Endidness::Little => self.i32_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i64` using the default endidness at the provided offset without altering the
    /// [`BinReader::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    fn i64_at(&self, offset: usize) -> Result<i64> {
        match self.endidness() {
            Endidness::Big => self.i64_be_at(offset),
            Endidness::Little => self.i64_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i128` using the default endidness at the provided offset without altering the
    /// [`BinReader::current_offset`]. If the current endidness is [`Endidness::Unknown`], then an
    /// error is returned.
    fn i128_at(&self, offset: usize) -> Result<i128> {
        match self.endidness() {
            Endidness::Big => self.i128_be_at(offset),
            Endidness::Little => self.i128_le_at(offset),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the current byte and then advances the cursor.
    fn next_u8(&self) -> Result<u8> {
        let byte = self.current_u8()?;
        self.advance_by(1)?;
        Ok(byte)
    }

    make_number_methods! {
        /// Gets the numendlong endian `numname` at the [`BinReader::current_offset`] and then
        /// advances it by `1`.
        fn next_numname_numend(&self) -> Result<_numname_> {
            let mut buf = [0; _numwidth_];
            self.next_bytes(&mut buf)?;
            Ok(_numname_::from_numend_bytes(buf))
        }
    }

    /// Gets the `u16` using the default endidness at the [`BinReader::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    fn next_u16(&self) -> Result<u16> {
        match self.endidness() {
            Endidness::Big => self.next_u16_be(),
            Endidness::Little => self.next_u16_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `u16` using the default endidness at the [`BinReader::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    fn next_u32(&self) -> Result<u32> {
        match self.endidness() {
            Endidness::Big => self.next_u32_be(),
            Endidness::Little => self.next_u32_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `u16` using the default endidness at the [`BinReader::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    fn next_u64(&self) -> Result<u64> {
        match self.endidness() {
            Endidness::Big => self.next_u64_be(),
            Endidness::Little => self.next_u64_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `u16` using the default endidness at the [`BinReader::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    fn next_u128(&self) -> Result<u128> {
        match self.endidness() {
            Endidness::Big => self.next_u128_be(),
            Endidness::Little => self.next_u128_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i8` at the [`BinReader::current_offset`] and then advances it by `1`. If the
    /// current endidness is [`Endidness::Unknown`], then an error is returned.
    fn next_i8(&self) -> Result<i8> {
        let mut buf = [0; 1];
        self.next_bytes(&mut buf)?;
        Ok(i8::from_be_bytes(buf))
    }

    /// Gets the `i16` using the default endidness at the [`BinReader::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    fn next_i16(&self) -> Result<i16> {
        match self.endidness() {
            Endidness::Big => self.next_i16_be(),
            Endidness::Little => self.next_i16_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i32` using the default endidness at the [`BinReader::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    fn next_i32(&self) -> Result<i32> {
        match self.endidness() {
            Endidness::Big => self.next_i32_be(),
            Endidness::Little => self.next_i32_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i64` using the default endidness at the [`BinReader::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    fn next_i64(&self) -> Result<i64> {
        match self.endidness() {
            Endidness::Big => self.next_i64_be(),
            Endidness::Little => self.next_i64_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    /// Gets the `i128` using the default endidness at the [`BinReader::current_offset`] and then
    /// advances it by `1`. If the current endidness is [`Endidness::Unknown`], then an error is
    /// returned.
    fn next_i128(&self) -> Result<i128> {
        match self.endidness() {
            Endidness::Big => self.next_i128_be(),
            Endidness::Little => self.next_i128_le(),
            Endidness::Unknown => Err(Error::UnknownEndidness),
        }
    }

    #[inline]
    fn slice_reader(&self, start: usize, end: usize) -> Result<SliceRefBinReader> {
        SliceRefBinReader::from_slice(self.range(start, end)?, self.endidness())
    }

    fn next_n_bytes_as_reader(&self, num_bytes: usize) -> Result<SliceRefBinReader> {
        let res = SliceRefBinReader::from_slice(
            self.subseq(self.current_offset(), num_bytes)?,
            self.endidness(),
        )?;
        self.advance_by(num_bytes as isize)?;
        Ok(res)
    }

    fn next_n_bytes_as_reader_retain_offset(&self, num_bytes: usize) -> Result<SliceRefBinReader> {
        let res = SliceRefBinReader::from_slice_with_offset(
            self.subseq(self.current_offset(), num_bytes)?,
            self.current_offset(),
            self.endidness(),
        )?;
        self.advance_by(num_bytes as isize)?;
        Ok(res)
    }

    #[inline]
    fn slice_reader_with_offset(
        &self,
        start: usize,
        offset: usize,
        end: usize,
    ) -> Result<SliceRefBinReader> {
        SliceRefBinReader::from_slice_with_offset(self.range(start, end)?, offset, self.endidness())
    }

    #[inline]
    fn slice_reader_retain_offset(&self, start: usize, end: usize) -> Result<SliceRefBinReader> {
        SliceRefBinReader::from_slice_with_offset(
            self.range(start, end)?,
            self.current_offset(),
            self.endidness(),
        )
    }
}

/// An implementor of [`OwnableBinReader`] owns the data contained within it. This means that they
/// can be built from more from more source (such as a [`bytes::Bytes`] instance or a file.
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
