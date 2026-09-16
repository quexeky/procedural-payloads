use core::marker::PhantomData;
use embedded_io::Read;
use zerocopy::TryFromBytes;

use crate::read::{error::ReadError, readable::Readable};

pub trait ReadableFrameField: Readable {}

impl<T: TryFromBytes> ReadableFrameField for T {}

pub struct FieldIterator<T: ReadableFrameField, R: Read>
where
    [(); T::SIZE]:,
{
    elements_remaining: usize,
    errored: bool,
    reader: R,
    _frame_type: PhantomData<T>,
}

impl<T: ReadableFrameField, R: Read> FieldIterator<T, R>
where
    [(); T::SIZE]:,
{
    pub fn new(num_fields: usize, reader: R) -> Self {
        Self {
            elements_remaining: num_fields,
            errored: false,
            reader,
            _frame_type: PhantomData,
        }
    }
    pub fn finish(self) -> Result<R, ReadError<R::Error>> {
        if self.errored || self.elements_remaining != 0 {
            return Err(ReadError::InsufficientData);
        }
        Ok(self.reader)
    }
}

impl<T: ReadableFrameField, R: Read> Iterator for FieldIterator<T, R>
where
    [(); T::SIZE]:,
{
    type Item = Result<T, ReadError<R::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.errored || self.elements_remaining == 0 {
            return None;
        }
        let mut buf = [0u8; T::SIZE];
        if let Err(e) = self.reader.read_exact(&mut buf) {
            self.errored = true;
            return Some(Err(ReadError::ReadExact(e)));
        }
        self.elements_remaining -= 1;
        match T::read(buf) {
            Ok(field) => Some(Ok(field)),
            Err(_) => {
                self.errored = true;
                Some(Err(ReadError::InvalidCast))
            }
        }
    }
}
