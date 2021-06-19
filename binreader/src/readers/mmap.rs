use crate::{BinReader, Endidness, OwnableBinReader, Result, SliceableBinReader};
use bytes::Bytes;
use fs3::FileExt as _;
use memmap2::{Mmap, MmapMut};
use std::{cell::Cell, fs::File, path::Path};

pub struct MmapBinReader {
    initial_offset: usize,
    position: Cell<usize>,
    map: Mmap,
    endidness: Endidness,
    maybe_mapped_file: Option<File>,
}

impl MmapBinReader {
    fn new(
        initial_offset: usize,
        map: Mmap,
        endidness: Endidness,
        maybe_mapped_file: Option<File>,
    ) -> Self {
        Self {
            initial_offset,
            position: Cell::new(0),
            map,
            endidness,
            maybe_mapped_file,
        }
    }

    fn adj_pos(&self, amt: isize) {
        let tmp = self.position.get() as isize;
        self.position.replace((tmp + amt) as usize);
    }
}

impl Drop for MmapBinReader {
    fn drop(&mut self) {
        if let Some(file) = &self.maybe_mapped_file {
            file.unlock().unwrap();
        }
    }
}

impl AsRef<[u8]> for MmapBinReader {
    fn as_ref(&self) -> &[u8] {
        self.map.as_ref()
    }
}

impl<'r> BinReader<'r> for MmapBinReader {
    #[inline]
    fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    #[inline]
    fn size(&self) -> usize {
        self.map.len()
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.size() - self.position.get()
    }

    #[inline]
    fn current_offset(&self) -> usize {
        self.initial_offset + self.position.get()
    }

    #[inline]
    fn endidness(&self) -> Endidness {
        self.endidness
    }

    #[inline]
    fn change_endidness(&mut self, endidness: Endidness) {
        self.endidness = endidness
    }

    #[inline]
    fn initial_offset(&self) -> usize {
        self.initial_offset
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

    fn u8_at(&self, offset: usize) -> Result<u8> {
        self.validate_offset(offset, 0)?;
        Ok(self.map[offset - self.initial_offset])
    }

    fn next_u8(&self) -> Result<u8> {
        self.validate_offset(self.current_offset(), 1)?;
        self.adj_pos(1);
        Ok(self.map[self.position.get() - 1])
    }

    fn from_slice_with_offset(
        slice: &[u8],
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        let mut mmap_mut = MmapMut::map_anon(slice.len())?;
        mmap_mut.copy_from_slice(slice);
        Ok(Self::new(
            initial_offset,
            mmap_mut.make_read_only()?,
            endidness,
            None,
        ))
    }
}

impl<'r> OwnableBinReader<'r> for MmapBinReader {
    fn from_file_with_offset<P: AsRef<Path>>(
        path: P,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        let file = File::open(path)?;
        file.try_lock_shared()?;
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self::new(initial_offset, mmap, endidness, Some(file)))
    }

    fn from_bytes_with_offset(
        bytes: Bytes,
        initial_offset: usize,
        endidness: Endidness,
    ) -> Result<Self> {
        Self::from_slice_with_offset(&bytes, initial_offset, endidness)
    }
}

impl<'r> SliceableBinReader<'r> for MmapBinReader {}

add_read! { MmapBinReader }
add_borrow! { MmapBinReader }
add_seek! { MmapBinReader }
add_bufread! { MmapBinReader }

#[cfg(feature = "nom")]
add_all_noms! { MmapBinReader }

#[cfg(test)]
mod tests {
    use super::*;
    test_reader! { MmapBinReader }
}
