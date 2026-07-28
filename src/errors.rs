use std::fmt::Debug;

use thiserror::Error;

use crate::Cursor;

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("insufficient data: requested {cursor:?}, view: {view:?}")]
    InsufficientData {
        cursor: Cursor,
        view: Cursor
    },

    #[error("I/O error: {0}")]
    IoError(#[source] Box<dyn std::error::Error>),
}

impl From<std::io::Error> for SourceError {
    fn from(e: std::io::Error) -> Self {
        SourceError::IoError(Box::new(e))
    }
}

pub type VResult<T> = Result<T, SourceError>;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("assertion failed: {msg}")]
    AssertionFailed {
        msg: String,
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

// use crate::cursor::Cursor;
// #[derive(Debug, Error)]
// pub enum CursorError {
//     #[error("cursors are disjoint: {a:?} and {b:?} do not overlap or touch")]
//     Disjoint {
//         a: Cursor,
//         b: Cursor,
//     },

//     #[error("subrange exceeds parent: parent {parent:?} cannot contain child offset {child_offset} size {child_size}")]
//     SubrangeExceeded {
//         parent: Cursor,
//         child_offset: usize,
//         child_size: usize,
//     },

//     #[error("numeric overflow when computing cursor with offset {offset} and size {size}")]
//     Overflow {
//         offset: usize,
//         size: usize,
//     },
// }

#[derive(Debug, Error)]
pub enum SpecError {
    #[error("read failure: {0}")]
    ReadFailed(#[from] SourceError),
    #[error("validation failed: {0}")]
    ValidationFailed(#[from] ValidationError),
}

pub type SResult<T> = Result<T, SpecError>;