use crate::{BinReader, Endidness, Result};

pub struct SliceRefReader<'r> {
    initial_offset: usize,
    position: usize,
    data: &'r [u8],
    endidness: Endidness,
}

impl<'r> SliceRefReader<'r> {
    #[inline]
    fn new(data: &'r [u8], initial_offset: usize, endidness: Endidness) -> Self {
        Self {
            initial_offset,
            position: 0,
            data,
            endidness,
        }
    }
}

impl<'r> AsRef<[u8]> for SliceRefReader<'r> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.data
    }
}

impl<'r, 'o> BinReader<'o> for SliceRefReader<'r>
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
        self.data.len() - self.position
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

    fn advance_by(&mut self, num_bytes: usize) -> Result<()> {
        self.validate_offset(self.position + num_bytes, 0)?;
        self.position += num_bytes;
        Ok(())
    }

    fn next_u8(&mut self) -> Result<u8> {
        self.validate_offset(self.current_offset(), 1)?;
        self.position += 1;
        Ok(self.data[self.position - 1])
    }
}

add_read! { SliceRefReader<'r>, 'r }
add_borrow! { SliceRefReader<'r>, 'r }
add_seek! { SliceRefReader<'r>, 'r }

//#[cfg(feature = "nom")]
//add_all_noms! { SliceRefReader<'r>, 'r }

pub struct SliceAsRefReader<R: AsRef<[u8]>> {
    initial_offset: usize,
    position: usize,
    data: R,
    endidness: Endidness,
}
impl<R: AsRef<[u8]>> SliceAsRefReader<R> {
    #[inline]
    fn _new(data: R, initial_offset: usize, endidness: Endidness) -> Self {
        Self {
            initial_offset,
            position: 0,
            data,
            endidness,
        }
    }
}

impl<R: AsRef<[u8]>> AsRef<[u8]> for SliceAsRefReader<R> {
    fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl<'r, R: AsRef<[u8]>> BinReader<'r> for SliceAsRefReader<R> {
    fn from_slice_with_offset(
        _slice: &'r [u8],
        _initial_offset: usize,
        _endidness: Endidness,
    ) -> Result<Self> {
        todo!();
    }

    #[inline]
    fn initial_offset(&self) -> usize {
        self.initial_offset
    }

    #[inline]
    fn size(&self) -> usize {
        self.data.as_ref().len()
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.data.as_ref().len() - self.position
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

    fn advance_by(&mut self, num_bytes: usize) -> Result<()> {
        self.validate_offset(self.position + num_bytes, 0)?;
        self.position += num_bytes;
        Ok(())
    }

    fn next_u8(&mut self) -> Result<u8> {
        self.validate_offset(self.current_offset(), 1)?;
        self.position += 1;
        Ok(self.data.as_ref()[self.position - 1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;

    #[test]
    fn basic_ref_test() {
        testing::basic_test_1::<SliceRefReader>();
    }

    #[test]
    fn basic_le_ref_test() {
        testing::basic_le_test::<SliceRefReader>();
    }

    #[test]
    fn basic_be_ref_test() {
        testing::basic_be_test::<SliceRefReader>();
    }
}
