
use std::ops::Add;

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub offset: usize,
    pub size: usize,
}

impl Cursor {
    pub fn new(offset: usize, size: usize) -> Self {
        Cursor { offset, size }
    }
    pub fn fromstart(size: usize) -> Self {
        Cursor { offset: 0, size }
    }
    pub fn end(self) -> usize {
        self.offset + self.size
    }

    /// Relative subrange: `other` is a cursor relative to this one.
    pub fn subrange(self, other: Cursor) -> Option<Self> {
        if other.end() > self.size {
            None
        } else {
            Some(other + self.offset)
        }
    }

    // Absolute checks
    pub fn is_subset(self, other: Self) -> bool {
        self.offset >= other.offset && self.end() <= other.end()
    }

    pub fn is_superset(self, other: Self) -> bool {
        other.is_subset(self)
    }

    pub fn shared(self, other: Self) -> Option<Self> {
        let start = self.offset.max(other.offset);
        let end = self.end().min(other.end());
        if start < end {
            Some(Cursor {
                offset: start,
                size: end - start,
            })
        } else {
            None
        }
    }

    pub fn join(self, other: Self) -> Option<Self> {
        if self.end() < other.offset || other.end() < self.offset {
            None
        } else {
            let start = self.offset.min(other.offset);
            let end = self.end().max(other.end());
            Some(Cursor {
                offset: start,
                size: end - start,
            })
        }
    }
}

impl Add<usize> for Cursor {
    type Output = Self;
    fn add(self, rhs: usize) -> Self {
        Cursor {
            offset: self.offset + rhs,
            size: self.size,
        }
    }
}