
use crate::array;
use crate::byte_source::ByteSource;
use crate::cursor::Cursor;
use crate::errors::{SourceError, ValidationError};
use crate::view::View;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpecError {
    #[error("read failure: {0}")]
    ReadFailed(#[from] SourceError),
    #[error("validation failed: {0}")]
    ValidationFailed(#[from] ValidationError),
}

pub trait Spec: Sized + std::fmt::Debug {
    type Params: Copy;
    fn read<'a, S: ByteSource>(
        data: &mut View<'a, S>,
        params: Self::Params,
    ) -> Result<Self, SpecError>;

    /// Convenience method: skip to `offset`, then read.
    fn read_offset<'a, S: ByteSource>(
        data: &mut View<'a, S>,
        offset: usize,
        params: Self::Params,
    ) -> Result<Self, SpecError> {
        data.skip(Cursor::fromstart(offset))?;
        Self::read(data, params)
    }
}

impl<T: Spec> Spec for Vec<T> {
    type Params = (usize, T::Params);

    fn read<'a, S: ByteSource>(
        data: &mut View<'a, S>,
        params: Self::Params,
    ) -> Result<Self, SpecError> {
        let mut output = Vec::with_capacity(params.0);
        for _ in 0..params.0 {
            output.push(T::read(data, params.1)?);
        }
        Ok(output)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Endianness {
    Big,
    Little,
}

macro_rules! impl_num_spec {
    ($name:ident, $ty:ty, $size:literal, $be:path, $le:path) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $name {
            pub value: $ty,
        }

        impl $name {
            pub fn LE<'a, S: ByteSource>(
                data: &mut View<'a, S>
            ) -> Result<Self, SpecError> {
                Self::read(data, Endianness::Little)
            }

            pub fn BE<'a, S: ByteSource>(
                data: &mut View<'a, S>
            ) -> Result<Self, SpecError> {
                Self::read(data, Endianness::Big)
            }
        }

        impl Spec for $name {
            type Params = Endianness;

            fn read<'a, S: ByteSource>(
                data: &mut View<'a, S>,
                params: Self::Params,
            ) -> Result<Self, SpecError> {
                let bytes = data.consume(Cursor::fromstart($size))?;
                let arr: [u8; $size] = bytes.try_into().unwrap();
                match params {
                    Endianness::Big => Ok($name {
                        value: $be(arr),
                    }),
                    Endianness::Little => Ok($name {
                        value: $le(arr),
                    }),
                }
            }
        }
    };
}


// --- experimental --- {
#[derive(Debug)]
pub struct Array<const N: usize, T>(pub [T; N]);

impl<const N: usize> Spec for Array<N, u8> {
    type Params = ();
    fn read<'a, S: ByteSource>(data: &mut View<'a, S>, _params: Self::Params) -> Result<Self, SpecError> {
        let arr = data.consume_n(N)?.try_into().unwrap();
        Ok(Array(arr))
    }
}

impl<const N: usize, T: Spec<Params = ()> + std::fmt::Debug> Spec for Array<N, T> {
    type Params = ();
    fn read<'a, S: ByteSource>(data: &mut View<'a, S>, _params: Self::Params) -> Result<Self, SpecError> {
        let arr = array!(T = data; N)?.try_into().unwrap();
        Ok(Array(arr))
    }
}
// } --- experimental ---

// ── Primitive types ──────────────────────────────────────
#[derive(Clone, Copy, Debug)]
pub struct U8 {
    pub value: u8,
}

impl Spec for U8 {
    type Params = ();

    fn read<'a, S: ByteSource>(
        data: &mut View<'a, S>,
        _params: Self::Params,
    ) -> Result<Self, SpecError> {
        let bytes = data.consume(Cursor::fromstart(1))?;
        let arr: [u8; 1] = bytes.try_into().unwrap();
        Ok(U8 {
            value: u8::from_be_bytes(arr),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct I8 {
    pub value: i8,
}

impl Spec for I8 {
    type Params = ();

    fn read<'a, S: ByteSource>(
        data: &mut View<'a, S>,
        _params: Self::Params,
    ) -> Result<Self, SpecError> {
        let bytes = data.consume(Cursor::fromstart(1))?;
        let arr: [u8; 1] = bytes.try_into().unwrap();
        Ok(I8 {
            value: i8::from_be_bytes(arr),
        })
    }
}

impl_num_spec!(U16, u16, 2, u16::from_be_bytes, u16::from_le_bytes);
impl_num_spec!(I16, i16, 2, i16::from_be_bytes, i16::from_le_bytes);
impl_num_spec!(U32, u32, 4, u32::from_be_bytes, u32::from_le_bytes);
impl_num_spec!(I32, i32, 4, i32::from_be_bytes, i32::from_le_bytes);
impl_num_spec!(U64, u64, 8, u64::from_be_bytes, u64::from_le_bytes);
impl_num_spec!(I64, i64, 8, i64::from_be_bytes, i64::from_le_bytes);
