use bytes::{BufMut as _, Bytes, BytesMut};
use std::{fs, io, path::Path};

pub fn bytes_from_file<P: AsRef<Path>>(path: P) -> io::Result<Bytes> {
    let capacity = fs::metadata(&path)?.len();
    let file = fs::File::open(path)?;
    bytes_from_bufread(io::BufReader::new(file), Some(capacity as usize))
}

pub fn bytes_from_bufread<R: io::BufRead>(
    mut reader: R,
    capacity: Option<usize>,
) -> io::Result<Bytes> {
    let mut bytes_mut = if let Some(size) = capacity {
        BytesMut::with_capacity(size)
    } else {
        BytesMut::new()
    };
    loop {
        let buf_len = {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                break Ok(Bytes::from(bytes_mut));
            }
            bytes_mut.put_slice(buf);
            buf.len()
        };
        reader.consume(buf_len);
    }
}
