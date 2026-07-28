
use crate::errors::SourceError;
use crate::view::View;
use crate::cursor::Cursor;

pub trait ByteSource: Clone {
    fn size(&self) -> usize;
    fn _unsafe_peek(&self, cursor: Cursor) -> &[u8];
    fn peek(&self, cursor: Cursor) -> Result<&[u8], SourceError> {
        self.check(cursor)?;
        Ok(self._unsafe_peek(cursor))
    }
    fn peek_n(&self, size: usize) -> Result<&[u8], SourceError> {
        self.peek(Cursor::fromstart(size))
    }
    fn check(&self, cursor: Cursor) -> Result<(), SourceError> {
        if cursor.end() < self.size() {
            Ok(())
        } else {
            Err(SourceError::InsufficientData {
                cursor,
                view: Cursor::fromstart(self.size())
            })
        }
    }
}

impl ByteSource for Vec<u8> {
    fn size(&self) -> usize {
        self.len()
    }

    fn _unsafe_peek(&self, cursor: Cursor) -> &[u8] {
        &self[cursor.offset..cursor.end()]
    }
}

impl<'a, S: ByteSource> ByteSource for View<'a, S> {
    fn size(&self) -> usize {
        self.cursor.size
    }

    fn _unsafe_peek(&self, cursor: Cursor) -> &[u8] {
        self.source._unsafe_peek(cursor + self.cursor.offset)
    }

    fn check(&self, cursor: Cursor) -> Result<(), SourceError> {
        if cursor.end() < self.size() {
            Ok(())
        } else {
            Err(SourceError::InsufficientData {
                cursor,
                view: self.cursor
            })
        }
    }
}