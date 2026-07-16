use crate::cursor::Cursor;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("insufficient data: requested {requested_size} bytes at offset {requested_offset}, but only {total_size} bytes available")]
    InsufficientData {
        requested_offset: usize,
        requested_size: usize,
        total_size: usize,
    },

    #[error("I/O error: {0}")]
    IoError(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl From<std::io::Error> for SourceError {
    fn from(e: std::io::Error) -> Self {
        SourceError::IoError(Box::new(e))
    }
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("assertion failed: {msg}")]
    AssertionFailed {
        msg: String,
        /// Byte offset where the violation was detected, if known.
        offset: Option<usize>,
    },
}

impl ValidationError {
    pub fn fail(msg: impl Into<String>) -> Self {
        ValidationError::AssertionFailed {
            msg: msg.into(),
            offset: None,
        }
    }

    pub fn fail_at(msg: impl Into<String>, offset: usize) -> Self {
        ValidationError::AssertionFailed {
            msg: msg.into(),
            offset: Some(offset),
        }
    }
}

#[derive(Debug, Error)]
pub enum CursorError {
    #[error("cursors are disjoint: {a:?} and {b:?} do not overlap or touch")]
    Disjoint {
        a: Cursor,
        b: Cursor,
    },

    #[error("subrange exceeds parent: parent {parent:?} cannot contain child offset {child_offset} size {child_size}")]
    SubrangeExceeded {
        parent: Cursor,
        child_offset: usize,
        child_size: usize,
    },

    #[error("numeric overflow when computing cursor with offset {offset} and size {size}")]
    Overflow {
        offset: usize,
        size: usize,
    },
}