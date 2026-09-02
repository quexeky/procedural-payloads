use core::marker::PhantomData;
use embedded_io::Read;

use crate::read::error::Error;

pub trait ReadableFrameField<const SIZE: usize>: TryFrom<[u8; SIZE]> {}

pub struct FieldIterator<'a, const SIZE: usize, T: ReadableFrameField<SIZE>, R: Read> {
    elements_remaining: usize,
    reader: &'a mut R,
    _frame_type: PhantomData<T>,
}

impl<'a, const SIZE: usize, T: ReadableFrameField<SIZE>, R: Read> FieldIterator<'a, SIZE, T, R> {
    pub fn new(num_fields: usize, reader: &'a mut R) -> Self {
        Self {
            elements_remaining: num_fields,
            reader,
            _frame_type: PhantomData,
        }
    }
    pub fn finish(self) {}
}

impl<'a, const SIZE: usize, T: ReadableFrameField<SIZE>, R: Read> Iterator
    for FieldIterator<'a, SIZE, T, R>
{
    type Item = Result<T, Error<R::Error, <T as TryFrom<[u8; SIZE]>>::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.elements_remaining == 0 {
            return None;
        }
        let mut buf = [0; SIZE];
        match self.reader.read_exact(&mut buf) {
            Ok(()) => {}
            Err(e) => return Some(Err(e.into())),
        };
        self.elements_remaining -= 1;
        let next = T::try_from(buf);
        Some(next.map_err(Error::TryFrom))
    }
}
