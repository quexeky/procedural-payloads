use core::marker::PhantomData;

use embedded_io::{Read, ReadExactError};

pub trait FrameField<const SIZE: usize>: AsRef<[u8]> + From<[u8; SIZE]> {}


pub struct FieldIterator<const SIZE: usize, T: FrameField<SIZE>, R: Read> {
    elements_remaining: usize,
    reader: R,
    _frame_type: PhantomData<T>,
}

impl<const SIZE: usize, T: FrameField<SIZE>, R: Read> FieldIterator<SIZE, T, R> {
    pub fn new(num_fields: usize, reader: R) -> Self {
        Self {
            elements_remaining: num_fields,
            reader,
            _frame_type: PhantomData,
        }
    }
}

impl<const SIZE: usize, T: FrameField<SIZE>, R: Read> Iterator for FieldIterator<SIZE, T, R> {
    type Item = Result<T, ReadExactError<R::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.elements_remaining == 0 {
            return None;
        }
        let mut buf = [0; SIZE];
        match self.reader.read_exact(&mut buf) {
            Ok(()) => {}
            Err(e) => return Some(Err(e)),
        };
        self.elements_remaining -= 1;
        Some(Ok(T::from(buf)))
    }
}