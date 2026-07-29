use either::{Left, Right, Either};

pub type Try<L, R, E> = Either<L, (R, E)>;
pub type TryResult<L, R, E> = Try<L, Result<R, E>, E>;
pub type TryString = Try<String, Vec<u8>, std::str::Utf8Error>;
pub fn get_string(data: &[u8]) -> TryString {
    let result = std::str::from_utf8(data);
    match result {
        Ok(val) => Left(val.to_string()),
        Err(err) => Right((Vec::from(data), err))
    }
}