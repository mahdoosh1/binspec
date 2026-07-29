
use std::cell::RefCell;

use crate::byte_source::ByteSource;
use crate::cursor::Cursor;
use crate::errors::VResult;

#[derive(Debug, Clone)]
pub struct View<'a, S: ByteSource> {
    pub source: &'a S,
    pub cursor: RefCell<Cursor>,
}

impl<'a, S: ByteSource> From<&'a S> for View<'a, S> {
    fn from(source: &'a S) -> Self {
        let size = source.size();
        View {
            source,
            cursor: RefCell::new(Cursor::fromstart(size)),
        }
    }
}

impl<'a, S: ByteSource> View<'a, S> {
    pub fn offset(&self) -> usize {
        self.cursor.borrow().offset
    }
    pub fn skip(&self, cursor: Cursor) -> VResult<()> {
        self.check(cursor)?;
        self.cursor.borrow_mut().offset += cursor.end();
        self.cursor.borrow_mut().size -= cursor.end();
        Ok(())
    }

    pub fn consume(&self, cursor: Cursor) -> VResult<&'a [u8]> {
        self.check(cursor)?;
        let output: Result<&'a [u8], crate::errors::SourceError> = self.source.peek(cursor + self.offset());
        self.skip(cursor)?;
        output
    }

    pub fn skip_n(&self, size: usize) -> VResult<()> {
        self.skip(Cursor::fromstart(size))
    }

    pub fn consume_n(&self, size: usize) -> VResult<&'a [u8]> {
        self.consume(Cursor::fromstart(size))
    }

    pub fn subview(&self, cursor: Cursor) -> VResult<Self> {
        self.check(cursor)?;
        let abs_cursor = cursor + self.offset();
        Ok(View {
            source: self.source,
            cursor: RefCell::new(abs_cursor),
        })
    }

    pub fn subview_n(&self, size: usize) -> VResult<Self> {
        self.subview(Cursor::fromstart(size))
    }

    pub fn consume_subview(&self, rel_cursor: Cursor) -> VResult<Self> {
        let output = self.subview(rel_cursor);
        self.skip(rel_cursor)?;
        output
    }

    pub fn consume_subview_n(&self, size: usize) -> VResult<Self> {
        self.consume_subview(Cursor::fromstart(size))
    }
}
