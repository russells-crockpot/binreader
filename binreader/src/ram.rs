use crate::{util::bytes_from_file, BinReader, Endidness, Error, OwnableBinReader, Result};
use bytes::Bytes;
use std::{borrow::Borrow, io, path::Path};

pub struct RandomAccessBinReader {
    initial_offset: usize,
    position: usize,
    data: Bytes,
    endidness: Endidness,
}

impl RandomAccessBinReader {
    #[inline]
    fn new(data: Bytes, initial_offset: usize, endidness: Endidness) -> Self {
        Self {
            initial_offset,
            position: 0,
            data,
            endidness,
        }
    }
}

impl AsRef<[u8]> for RandomAccessBinReader {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl<'r> BinReader<'r> for RandomAccessBinReader {
    #[inline]
    fn from_slice(slice: &[u8], initial_offset: usize, endidness: Endidness) -> Result<Self> {
        Self::from_bytes(Bytes::copy_from_slice(slice), initial_offset, endidness)
    }

    #[inline]
    fn initial_offset(&self) -> usize {
        self.initial_offset
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    #[inline]
    fn size(&self) -> usize {
        self.data.len()
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.size() - self.position
    }

    #[inline]
    fn current_offset(&self) -> usize {
        self.position + self.initial_offset
    }

    #[inline]
    fn endidness(&self) -> Endidness {
        self.endidness
    }

    fn advance_to(&mut self, offset: usize) -> Result<()> {
        self.validate_offset(offset, 0)?;
        self.position = offset;
        Ok(())
    }

    fn advance_by(&mut self, bytes: usize) -> Result<()> {
        self.validate_offset(self.current_offset() + bytes, 0)?;
        self.position += bytes;
        Ok(())
    }

    fn next_u8(&mut self) -> Result<u8> {
        self.validate_offset(self.current_offset(), 1)?;
        self.position += 1;
        Ok(self.data.as_ref()[self.position - 1])
    }

    fn next_n_bytes(&mut self, num_bytes: usize) -> Result<Bytes> {
        self.validate_offset(self.current_offset(), num_bytes)?;
        self.position += num_bytes;
        Ok(self
            .data
            .slice(self.current_offset()..self.current_offset() + num_bytes))
    }
}

impl<'r> OwnableBinReader<'r> for RandomAccessBinReader {
    #[inline]
    fn from_file<P: AsRef<Path>>(
        path: P,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Ok(Self::new(bytes_from_file(path)?, initial_offset, endidness))
    }

    #[inline]
    fn from_bytes(bytes: Bytes, initial_offset: usize, endidness: Endidness) -> Result<Self> {
        Ok(Self::new(bytes, initial_offset, endidness))
    }
}

add_read! { RandomAccessBinReader }
add_borrow! { RandomAccessBinReader }
add_seek! { RandomAccessBinReader }

#[cfg(feature = "nom")]
add_all_noms! { RandomAccessBinReader }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;

    #[test]
    fn basic_test() {
        testing::basic_test_1::<RandomAccessBinReader>();
    }

    #[test]
    fn basic_le_test() {
        testing::basic_le_test::<RandomAccessBinReader>();
    }

    #[test]
    fn basic_be_test() {
        testing::basic_be_test::<RandomAccessBinReader>();
    }
}
