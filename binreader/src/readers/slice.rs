use crate::{BinReader, Endidness, Result};
use std::cell::Cell;

/// A [`SliceRefBinReader`]
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

    fn adj_pos(&self, amt: isize) {
        let tmp = self.position.get() as isize;
        self.position.replace((tmp + amt) as usize);
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
    fn get_remaining(&self) -> Result<&[u8]> {
        Ok(&self.data[self.position.get()..])
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

    #[inline]
    fn change_endidness(&mut self, endidness: Endidness) {
        self.endidness = endidness
    }

    fn advance_to(&self, offset: usize) -> Result<()> {
        self.validate_offset(offset, 0)?;
        self.position.replace(offset - self.initial_offset);
        Ok(())
    }

    fn advance_by(&self, num_bytes: isize) -> Result<()> {
        self.validate_offset((self.current_offset() as isize + num_bytes) as usize, 0)?;
        self.adj_pos(num_bytes);
        Ok(())
    }

    fn next_u8(&self) -> Result<u8> {
        self.validate_offset(self.current_offset(), 1)?;
        self.adj_pos(1);
        Ok(self.data[self.position.get() - 1])
    }
}

impl<'r> SliceableBinReader<'r> for SliceRefBinReader<'r> {}

add_read! { SliceRefBinReader<'r>, 'r }
add_borrow! { SliceRefBinReader<'r>, 'r }
add_seek! { SliceRefBinReader<'r>, 'r }
add_bufread! { SliceRefBinReader<'r>, 'r }

pub trait SliceableBinReader<'r>: BinReader<'r> {
    #[inline]
    fn slice_reader(&self, start: usize, end: usize) -> Result<SliceRefBinReader> {
        SliceRefBinReader::from_slice(self.range(start, end)?, self.endidness())
    }

    #[inline]
    fn next_n_bytes_as_reader(&self, num_bytes: usize) -> Result<SliceRefBinReader> {
        let res = SliceRefBinReader::from_slice(
            self.subseq(self.current_offset(), num_bytes)?,
            self.endidness(),
        )?;
        self.advance_by(num_bytes as isize)?;
        Ok(res)
    }

    #[inline]
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

#[cfg(test)]
mod tests {
    use super::*;
    test_reader! { SliceRefBinReader }
}
