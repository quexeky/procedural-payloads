#[derive(Debug)]
pub enum WriteError<E> {
    TooMuchPlannedData,
    InsufficientDataWritten,
    ExcessData,
    Other(E),
}

impl<E: embedded_io::Error> From<E> for WriteError<E> {
    fn from(value: E) -> Self {
        WriteError::Other(value)
    }
}
