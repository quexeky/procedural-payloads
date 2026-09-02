use embedded_io::ReadExactError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error<E: embedded_io::Error, F> {
    ReadExact(#[from] ReadExactError<E>),
    GenericRead(#[from] E),
    TryFrom(F)
}
