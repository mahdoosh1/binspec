
use crate::assert_spec_eq;
use crate::byte_source::ByteSource;
use crate::errors::SResult;
use crate::view::View;
use either;

pub trait Spec: Sized + std::fmt::Debug {
    type Params: Copy;

    fn read_all<S: ByteSource>(data: &S, params: Self::Params) -> SResult<(Self, usize)>;

    fn read_from_view<'a, S: ByteSource>(view: &mut View<'a, S>, params: Self::Params) -> SResult<Self> {
        let (output, size) = Self::read_all(view, params)?;
        view.skip_n(size)?;
        Ok(output)
    }

    fn size<S: ByteSource>(
        data: &S,
        params: Self::Params,
    ) -> SResult<usize> {
        Ok(Self::read_all(data, params)?.1)
    }

    fn read<S: ByteSource>(
        data: &S,
        params: Self::Params,
    ) -> SResult<Self> {
        let all = Self::read_all(data, params)?;
        let size = Self::size(data, params)?;
        assert_spec_eq!(all.1, size);
        Ok(all.0)
    }

    fn read_all_at<S: ByteSource>(
        data: &S,
        offset: usize,
        params: Self::Params,
    ) -> SResult<(Self, usize)> {
        let mut view = View::from(data);
        view.skip_n(offset)?;
        Self::read_all(&view, params)
    }

    fn size_at<S: ByteSource>(
        data: &S,
        offset: usize,
        params: Self::Params,
    ) -> SResult<usize> {
        let mut view = View::from(data);
        view.skip_n(offset)?;
        Self::size(&view, params)
    }

    fn read_at<S: ByteSource>(
        data: &S,
        offset: usize,
        params: Self::Params,
    ) -> SResult<Self> {
        let mut view = View::from(data);
        view.skip_n(offset)?;
        Self::read(&view, params)
    }

}

impl<T: Spec> Spec for Vec<T> {
    type Params = (usize, T::Params);
    
    fn read_all<S: ByteSource>(
        data: &S,
        params: Self::Params,
    ) -> SResult<(Self, usize)> {
        
        let mut output = Vec::with_capacity(params.0);
        let mut acc = 0;
        for _ in 0..params.0 {
            let (spec, size) = T::read_all(data, params.1)?;
            acc += size;
            output.push(spec);
        }
        Ok((output, acc))
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
            #[allow(non_snake_case)]
            pub fn LE<'a, S: ByteSource>(
                data: &mut View<'a, S>
            ) -> SResult<Self> {
                Self::read(data, Endianness::Little)
            }
            
            #[allow(non_snake_case)]
            pub fn BE<'a, S: ByteSource>(
                data: &mut View<'a, S>
            ) -> SResult<Self> {
                Self::read(data, Endianness::Big)
            }
        }

        impl Spec for $name {
            type Params = Endianness;

            fn read_all<S: ByteSource>(
                data: &S,
                params: Self::Params,
            ) -> SResult<(Self, usize)> {
                let arr: [u8; $size] = data.peek_n($size)?.try_into().unwrap();
                Ok((match params {
                    Endianness::Big => $name {
                        value: $be(arr),
                    },
                    Endianness::Little => $name {
                        value: $le(arr),
                    },
                }, $size))
            }
        }
    };
}


// --- experimental --- {
// use crate::array
// #[derive(Debug)]
// pub struct Array<const N: usize, T>(pub [T; N]);

// impl<const N: usize> Spec for Array<N, u8> {
//     type Params = ();
//     fn read<'a, S: ByteSource>(data: &mut View<'a, S>, _params: Self::Params) -> SResult<Self> {
//         let arr = data.consume_n(N)?.try_into().unwrap();
//         Ok(Array(arr))
//     }
// }

// impl<const N: usize, T: Spec<Params = ()> + std::fmt::Debug> Spec for Array<N, T> {
//     type Params = ();
//     fn read<'a, S: ByteSource>(data: &mut View<'a, S>, _params: Self::Params) -> SResult<Self> {
//         let arr = array!(T = data; N)?.try_into().unwrap();
//         Ok(Array(arr))
//     }
// }
// } --- experimental ---

#[derive(Clone, Copy, Debug)]
pub struct U8 {
    pub value: u8,
}

impl Spec for U8 {
    type Params = ();

    fn read_all<S: ByteSource>(
         data: &S,
        _params: Self::Params,
    ) -> SResult<(Self, usize)> {
        let arr: [u8; 1] = data.peek_n(1)?.try_into().unwrap();
        Ok((U8 {
            value: u8::from_be_bytes(arr),
        }, 1))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct I8 {
    pub value: i8,
}

impl Spec for I8 {
    type Params = ();

    fn read_all<S: ByteSource>(
         data: &S,
        _params: Self::Params,
    ) -> SResult<(Self, usize)> {
        let arr: [u8; 1] = data.peek_n(1)?.try_into().unwrap();
        Ok((I8 {
            value: i8::from_be_bytes(arr),
        }, 1))
    }
}

impl_num_spec!(U16, u16, 2, u16::from_be_bytes, u16::from_le_bytes);
impl_num_spec!(I16, i16, 2, i16::from_be_bytes, i16::from_le_bytes);
impl_num_spec!(U32, u32, 4, u32::from_be_bytes, u32::from_le_bytes);
impl_num_spec!(I32, i32, 4, i32::from_be_bytes, i32::from_le_bytes);
impl_num_spec!(U64, u64, 8, u64::from_be_bytes, u64::from_le_bytes);
impl_num_spec!(I64, i64, 8, i64::from_be_bytes, i64::from_le_bytes);

pub type Try<L, R, E> = either::Either<L, (Result<R, E>, E)>;
#[allow(type_alias_bounds)]
pub type TrySpec<L: Spec, R: Spec> = Try<L, R, crate::errors::SpecError>;

pub fn try_spec<'a, S: ByteSource, L: Spec, R: Spec>(
    l: impl FnOnce(&mut View<'a, S>) -> SResult<L>,
    r: impl FnOnce(&mut View<'a, S>) -> SResult<R>,
    data: &mut View<'a, S>
) -> TrySpec<L, R> {
    let mut left_data = data.clone();
    match l(&mut left_data) {
        Ok(left) => {
            data.cursor = left_data.cursor;
            either::Either::Left(left)
        },
        Err(error) => {
            either::Either::Right((r(data),error))
        }
    }
}