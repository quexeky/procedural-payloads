use embedded_io::ReadExactError;
use thiserror::Error;
use zerocopy::ConvertError;

#[derive(Error, Debug)]
pub enum Error<E: embedded_io::Error> {
    ReadExact(#[from] ReadExactError<E>),
    InvalidCast,
}

impl<A, S, V, E: embedded_io::Error> From<ConvertError<A, S, V>> for Error<E> {
    fn from(_value: ConvertError<A, S, V>) -> Self {
        Error::InvalidCast
    }
}
