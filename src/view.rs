
use crate::byte_source::ByteSource;
use crate::cursor::Cursor;
use crate::errors::VResult;

#[derive(Debug, Clone, Copy)]
pub struct View<'a, S: ByteSource> {
    pub source: &'a S,
    pub cursor: Cursor,
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
    pub fn skip(&mut self, cursor: Cursor) -> VResult<()> {
        self.check(cursor)?;
        self.cursor.offset += cursor.end();
        self.cursor.size -= cursor.end();
        Ok(())
    }

    pub fn consume(&mut self, cursor: Cursor) -> VResult<&[u8]> {
        self.check(cursor)?;
        let output = self.source.peek(cursor + self.cursor.offset);
        self.skip(cursor)?;
        output
    }

    pub fn skip_n(&mut self, size: usize) -> VResult<()> {
        self.skip(Cursor::fromstart(size))
    }

    pub fn consume_n(&mut self, size: usize) -> VResult<&[u8]> {
        self.consume(Cursor::fromstart(size))
    }

    pub fn subview(&self, cursor: Cursor) -> VResult<Self> {
        self.check(cursor)?;
        let abs_cursor = cursor + self.cursor.offset;
        Ok(View {
            source: self.source,
            cursor: abs_cursor,
        })
    }

    pub fn subview_n(&self, size: usize) -> VResult<Self> {
        self.subview(Cursor::fromstart(size))
    }

    pub fn consume_subview(&mut self, rel_cursor: Cursor) -> VResult<Self> {
        let output = self.subview(rel_cursor);
        self.skip(rel_cursor)?;
        output
    }

    pub fn consume_subview_n(&mut self, size: usize) -> VResult<Self> {
        self.consume_subview(Cursor::fromstart(size))
    }
}
