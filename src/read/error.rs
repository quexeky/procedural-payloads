use embedded_io::ReadExactError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReadError<E: embedded_io::Error> {
    ReadExact(#[from] ReadExactError<E>),
    InsufficientData,
    InvalidCast,
    InvalidCrc {
        calculated: u32,
        read: u32
    }
}
