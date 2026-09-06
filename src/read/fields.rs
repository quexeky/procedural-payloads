use core::marker::PhantomData;
use embedded_io::Read;
use zerocopy::TryFromBytes;

use crate::read::{error::Error, readable::Readable};

pub trait ReadableFrameField: Readable {}

impl<T: TryFromBytes> ReadableFrameField for T {}

pub struct FieldIterator<T: ReadableFrameField, R: Read>
where
    [(); T::SIZE]:,
{
    elements_remaining: usize,
    reader: R,
    _frame_type: PhantomData<T>,
}

impl<'a, T: ReadableFrameField, R: Read> FieldIterator<T, R>
where
    [(); T::SIZE]:,
{
    pub fn new(num_fields: usize, reader: R) -> Self {
        Self {
            elements_remaining: num_fields,
            reader,
            _frame_type: PhantomData,
        }
    }
    pub fn finish(self) {}
}

impl<'a, T: ReadableFrameField, R: Read> Iterator for FieldIterator<T, R>
where
    [(); T::SIZE]:,
{
    type Item = Result<T, Error<R::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.elements_remaining == 0 {
            return None;
        }
        let mut buf = [0u8; T::SIZE];
        match self.reader.read_exact(&mut buf) {
            Ok(()) => {}
            Err(e) => return Some(Err(e.into())),
        };
        self.elements_remaining -= 1;
        let next = T::read::<R>(buf);
        Some(next.map_err(|_| Error::InvalidCast))
    }
}
