use crate::{util::bytes_from_file, BinReader, Endidness, Error, OwnableBinReader, Result};

use bytes::{Buf as _, Bytes};
use std::path::Path;

pub struct ConsumingBinReader {
    initial_offset: usize,
    size: usize,
    data: Bytes,
    endidness: Endidness,
}

impl ConsumingBinReader {
    #[inline]
    fn new(data: Bytes, initial_offset: usize, endidness: Endidness) -> Self {
        Self {
            initial_offset,
            size: data.len(),
            data,
            endidness,
        }
    }
}

impl AsRef<[u8]> for ConsumingBinReader {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl From<ConsumingBinReader> for Bytes {
    #[inline]
    fn from(reader: ConsumingBinReader) -> Self {
        reader.data
    }
}

impl<'r> BinReader<'r> for ConsumingBinReader {
    #[inline]
    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline]
    fn size(&self) -> usize {
        self.size
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.data.remaining()
    }

    #[inline]
    fn endidness(&self) -> Endidness {
        self.endidness
    }

    #[inline]
    fn initial_offset(&self) -> usize {
        self.current_offset()
    }
    #[inline]
    fn lower_offset_limit(&self) -> usize {
        self.initial_offset
    }

    #[inline]
    fn current_offset(&self) -> usize {
        (self.size - self.data.remaining()) + self.initial_offset
    }

    fn bytes_at(&self, offset: usize, buf: &mut [u8]) -> Result<()> {
        self.validate_offset(offset, buf.len())?;
        for i in 0..buf.len() {
            buf[i] = self.data[offset + i];
        }
        Ok(())
    }

    fn next_u8(&mut self) -> Result<u8> {
        if self.is_empty() {
            Err(Error::NoMoreData)
        } else {
            Ok(self.data.get_u8())
        }
    }

    fn u8_at(&self, offset: usize) -> Result<u8> {
        self.validate_offset(offset, 0)?;
        Ok(self.data[offset - self.current_offset()])
    }

    fn advance_to(&mut self, offset: usize) -> Result<()> {
        self.validate_offset(offset, 0)?;
        self.advance_by(offset - self.current_offset())?;
        Ok(())
    }

    fn advance_by(&mut self, bytes: usize) -> Result<()> {
        self.validate_offset(self.current_offset() + bytes, 0)?;
        self.data.advance(bytes);
        Ok(())
    }

    fn next_n_bytes(&mut self, num_bytes: usize) -> Result<Bytes> {
        self.validate_offset(self.current_offset(), num_bytes)?;
        Ok(self.data.split_to(num_bytes))
    }

    fn from_slice_with_offset(
        slice: &'r [u8],
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Self::from_bytes_with_offset(Bytes::copy_from_slice(slice), initial_offset, endidness)
    }
}

impl<'r> OwnableBinReader<'r> for ConsumingBinReader {
    fn from_file_with_offset<P: AsRef<Path>>(
        path: P,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Ok(Self::new(bytes_from_file(path)?, initial_offset, endidness))
    }

    fn from_bytes_with_offset(
        bytes: Bytes,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Ok(Self::new(bytes, initial_offset, endidness))
    }
}

add_read! { ConsumingBinReader }
add_borrow! { ConsumingBinReader }

#[cfg(feature = "nom")]
add_all_noms! { ConsumingBinReader }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;

    #[test]
    fn basic_test() {
        testing::basic_test_1::<ConsumingBinReader>();
    }

    #[test]
    fn basic_le_test() {
        testing::basic_le_test::<ConsumingBinReader>();
    }

    #[test]
    fn basic_be_test() {
        testing::basic_be_test::<ConsumingBinReader>();
    }
}
