use embedded_io::ReadExactError;
use thiserror::Error;
use zerocopy::ConvertError;

#[derive(Error, Debug)]
pub enum ReadError<E: embedded_io::Error> {
    ReadExact(#[from] ReadExactError<E>),
    InvalidCast,
    InvalidCrc {
        calculated: u32,
        read: u32
    }
}

impl<A, S, V, E: embedded_io::Error> From<ConvertError<A, S, V>> for ReadError<E> {
    fn from(_value: ConvertError<A, S, V>) -> Self {
        ReadError::InvalidCast
    }
}
