#[derive(Debug)]
pub enum Error<E> {
    TooMuchPlannedData,
    InsufficientDataWritten,
    ExcessData,
    Other(E)
}

impl<E: embedded_io::Error> From<E> for Error<E> {
    fn from(value: E) -> Self {
        Error::Other(value)
    }
}