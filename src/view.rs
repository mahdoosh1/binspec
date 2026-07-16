
use crate::byte_source::ByteSource;
use crate::cursor::Cursor;
use crate::errors::SourceError;

#[derive(Clone, Copy)]
pub struct View<'a, S: ByteSource> {
    pub(crate) source: &'a S,
    pub(crate) cursor: Cursor,
}

impl<'a, S: ByteSource> From<&'a S> for View<'a, S> {
    fn from(source: &'a S) -> Self {
        let size = source.size();
        View {
            source,
            cursor: Cursor::fromstart(size),
        }
    }
}

impl<'a, S: ByteSource> View<'a, S> {
    /// Check that a relative cursor fits inside the current view window.
    pub fn check(&self, rel_cursor: Cursor) -> Result<(), SourceError> {
        let abs_offset = self.cursor.offset + rel_cursor.offset;
        if !self
            .cursor
            .is_superset(Cursor {
                offset: abs_offset,
                size: rel_cursor.size,
            })
        {
            Err(SourceError::InsufficientData {
                requested_offset: abs_offset,
                requested_size: rel_cursor.size,
                total_size: self.cursor.size,
            })
        } else {
            Ok(())
        }
    }

    /// Peek at bytes without advancing the view.
    pub fn peek(&self, cursor: Cursor) -> Result<Vec<u8>, SourceError> {
        self.check(cursor)?;
        let mut output = vec![0u8; cursor.size];
        let read_offset = self.cursor.offset + cursor.offset;
        self.source.read(read_offset, &mut output)?;
        Ok(output)
    }

    /// Advance the view past a region described by a relative cursor.
    pub fn skip(&mut self, cursor: Cursor) -> Result<(), SourceError> {
        self.check(cursor)?;
        self.cursor.offset += cursor.end();
        self.cursor.size -= cursor.end();
        Ok(())
    }

    /// Read bytes and advance the view.
    pub fn consume(&mut self, cursor: Cursor) -> Result<Vec<u8>, SourceError> {
        let output = self.peek(cursor)?;
        self.skip(cursor)?;
        Ok(output)
    }

    pub fn peek_n(&self, size: usize) -> Result<Vec<u8>, SourceError> {
        self.peek(Cursor::fromstart(size))
    }

    pub fn skip_n(&mut self, size: usize) -> Result<(), SourceError> {
        self.skip(Cursor::fromstart(size))
    }

    pub fn consume_n(&mut self, size: usize) -> Result<Vec<u8>, SourceError> {
        self.consume(Cursor::fromstart(size))
    }

    /// Create a sub-view that is a window inside the current view.
    pub fn subview(&self, cursor: Cursor) -> Result<Self, SourceError> {
        let abs_offset = self.cursor.offset + cursor.offset;
        let sub_cursor = self
            .cursor
            .subrange(cursor)
            .ok_or_else(|| SourceError::InsufficientData {
                requested_offset: abs_offset,
                requested_size: cursor.size,
                total_size: self.cursor.size,
            })?;
        Ok(View {
            source: self.source,
            cursor: sub_cursor,
        })
    }
}
