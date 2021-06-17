use crate::{BinReader, Endidness};
use std::marker::PhantomData;

pub struct BorrowedBinReaderIter<'a, 'r, B>
where
    'r: 'a,
    B: BinReader<'r>,
{
    reader: &'a B,
    cursor: usize,
    length: usize,
    endidness_override: Option<Endidness>,
    _marker: Option<&'r PhantomData<bool>>,
}

impl<'a, 'r, B> BorrowedBinReaderIter<'a, 'r, B>
where
    'r: 'a,
    B: BinReader<'r>,
{
    pub(crate) fn new(reader: &'a B, endidness_override: Option<Endidness>) -> Self {
        Self {
            reader,
            cursor: 0,
            length: reader.remaining(),
            endidness_override,
            _marker: None,
        }
    }

    pub fn endidness(&self) -> Endidness {
        #[allow(clippy::or_fun_call)]
        self.endidness_override.unwrap_or(self.reader.endidness())
    }
}
