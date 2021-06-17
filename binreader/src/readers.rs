mod random_access;
mod slice;

pub use random_access::RandomAccessBinReader;
pub use slice::{SliceRefBinReader, SliceableReader};

#[cfg(feature = "memmap")]
mod mmap;
#[cfg(feature = "memmap")]
pub use mmap::MmapBinReader;
