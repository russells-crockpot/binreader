use crate::{
    util::bytes_from_file, BinReader, Endidness, OwnableBinReader, Result, SliceableBinReader,
};
use bytes::Bytes;
use std::{cell::Cell, path::Path};

pub struct RandomAccessBinReader {
    initial_offset: usize,
    position: Cell<usize>,
    data: Bytes,
    endidness: Endidness,
}

impl RandomAccessBinReader {
    #[inline]
    fn new(data: Bytes, initial_offset: usize, endidness: Endidness) -> Self {
        Self {
            initial_offset,
            position: Cell::new(0),
            data,
            endidness,
        }
    }

    fn adj_pos(&self, amt: isize) {
        let tmp = self.position.get() as isize;
        self.position.replace((tmp + amt) as usize);
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
    fn from_slice_with_offset(
        slice: &[u8],
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Self::from_bytes_with_offset(Bytes::copy_from_slice(slice), initial_offset, endidness)
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
        self.size() - self.position.get()
    }

    #[inline]
    fn current_offset(&self) -> usize {
        self.position.get() + self.initial_offset
    }

    #[inline]
    fn endidness(&self) -> Endidness {
        self.endidness
    }

    fn advance_to(&self, offset: usize) -> Result<()> {
        self.validate_offset(offset, 0)?;
        self.position.replace(offset);
        Ok(())
    }

    fn advance_by(&self, bytes: isize) -> Result<()> {
        self.validate_offset((self.position.get() as isize + bytes) as usize, 0)?;
        self.adj_pos(bytes);
        Ok(())
    }

    fn next_u8(&self) -> Result<u8> {
        self.validate_offset(self.current_offset(), 1)?;
        self.adj_pos(1);
        Ok(self.data.as_ref()[self.position.get() - 1])
    }

    fn next_n_bytes(&self, num_bytes: usize) -> Result<Bytes> {
        self.validate_offset(self.current_offset(), num_bytes)?;
        self.adj_pos(num_bytes as isize);
        Ok(self
            .data
            .slice(self.current_offset()..self.current_offset() + num_bytes))
    }
}

impl<'r> OwnableBinReader<'r> for RandomAccessBinReader {
    #[inline]
    fn from_file_with_offset<P: AsRef<Path>>(
        path: P,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Ok(Self::new(bytes_from_file(path)?, initial_offset, endidness))
    }

    #[inline]
    fn from_bytes_with_offset(
        bytes: Bytes,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Ok(Self::new(bytes, initial_offset, endidness))
    }
}

impl<'r> SliceableBinReader<'r> for RandomAccessBinReader {}

add_read! { RandomAccessBinReader }
add_borrow! { RandomAccessBinReader }
add_seek! { RandomAccessBinReader }
add_bufread! { RandomAccessBinReader }

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
