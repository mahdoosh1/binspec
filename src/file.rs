use std::{fs::File, io, path::Path, sync::Arc};
use memmap2::Mmap;

use crate::byte_source::ByteSource;
use crate::cursor::Cursor;

#[derive(Clone)]
pub struct MmapByteSource {
    mmap: Arc<Mmap>,
    len: usize,
}

impl MmapByteSource {
    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        Self::from_file(file)
    }

    pub fn from_file(file: File) -> io::Result<Self> {
        let len = file.metadata()?.len() as usize;
        // SAFETY: file is opened read‑only; we never mutate it.
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(MmapByteSource {
            mmap: Arc::new(mmap),
            len,
        })
    }
}

impl ByteSource for MmapByteSource {
    fn size(&self) -> usize {
        self.len
    }

    fn _unsafe_peek(&self, cursor: Cursor) -> &[u8] {
        let end = cursor.end();
        if end > self.len {
            panic!("peek out of bounds");
        }
        // `mmap` derefs to `&[u8]`, so slicing is O(1) and zero‑copy.
        &self.mmap[cursor.offset..end]
    }
}