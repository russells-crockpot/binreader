use crate::{BinReader, Endidness, Result};
use std::cell::Cell;

pub struct SliceRefBinReader<'r> {
    initial_offset: usize,
    position: Cell<usize>,
    data: &'r [u8],
    endidness: Endidness,
}

impl<'r> SliceRefBinReader<'r> {
    #[inline]
    fn new(data: &'r [u8], initial_offset: usize, endidness: Endidness) -> Self {
        Self {
            initial_offset,
            position: Cell::new(0),
            data,
            endidness,
        }
    }

    fn inc_pos(&self, amt: usize) {
        let tmp = self.position.get();
        self.position.replace(tmp + amt);
    }
}

impl<'r> AsRef<[u8]> for SliceRefBinReader<'r> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.data
    }
}

impl<'r, 'o> BinReader<'o> for SliceRefBinReader<'r>
where
    'o: 'r,
{
    fn from_slice_with_offset(
        slice: &'o [u8],
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Ok(Self::new(slice, initial_offset, endidness))
    }

    #[inline]
    fn initial_offset(&self) -> usize {
        self.initial_offset
    }

    #[inline]
    fn size(&self) -> usize {
        self.data.len()
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.data.len() - self.position.get()
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

    fn advance_by(&self, num_bytes: usize) -> Result<()> {
        self.validate_offset(self.position.get() + num_bytes, 0)?;
        self.inc_pos(num_bytes);
        Ok(())
    }

    fn next_u8(&self) -> Result<u8> {
        self.validate_offset(self.current_offset(), 1)?;
        self.inc_pos(1);
        Ok(self.data[self.position.get() - 1])
    }
}

impl<'r> SliceableReader<'r> for SliceRefBinReader<'r> {}

add_read! { SliceRefBinReader<'r>, 'r }
add_borrow! { SliceRefBinReader<'r>, 'r }
add_seek! { SliceRefBinReader<'r>, 'r }
add_bufread! { SliceRefBinReader<'r>, 'r }

pub trait SliceableReader<'r>: BinReader<'r> {
    #[inline]
    fn slice_reader(&self, start: usize, end: usize) -> Result<SliceRefBinReader> {
        SliceRefBinReader::from_slice(self.range(start, end)?, self.endidness())
    }

    #[inline]
    fn next_n_bytes_as_reader(&self, num_bytes: usize) -> Result<SliceRefBinReader> {
        SliceRefBinReader::from_slice(
            self.subseq(self.current_offset(), num_bytes)?,
            self.endidness(),
        )
    }

    #[inline]
    fn next_n_bytes_as_reader_retain_offset(
        &mut self,
        num_bytes: usize,
    ) -> Result<SliceRefBinReader> {
        SliceRefBinReader::from_slice_with_offset(
            self.subseq(self.current_offset(), num_bytes)?,
            self.current_offset(),
            self.endidness(),
        )
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;

    #[test]
    fn basic_ref_test() {
        testing::basic_test_1::<SliceRefBinReader>();
    }

    #[test]
    fn basic_le_ref_test() {
        testing::basic_le_test::<SliceRefBinReader>();
    }

    #[test]
    fn basic_be_ref_test() {
        testing::basic_be_test::<SliceRefBinReader>();
    }
}
