
use crate::errors::SourceError;

pub trait ByteSource {
    fn size(&self) -> usize;
    fn read(&self, offset: usize, buf: &mut [u8]) -> Result<(), SourceError>;
}

impl ByteSource for Vec<u8> {
    fn size(&self) -> usize {
        self.len()
    }
    fn read(&self, offset: usize, buf: &mut [u8]) -> Result<(), SourceError> {
        let end = offset + buf.len();
        if end > self.len() {
            return Err(SourceError::InsufficientData {
                requested_offset: offset,
                requested_size: buf.len(),
                total_size: self.len(),
            });
        }
        buf.copy_from_slice(&self[offset..end]);
        Ok(())
    }
}
