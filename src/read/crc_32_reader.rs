use crc32fast::Hasher;
use embedded_io::{ErrorType, Read};

pub struct Crc32Reader<'a, R: Read> {
    reader: &'a mut R,
    hasher: Hasher,
}

impl<'a, R: Read> ErrorType for Crc32Reader<'a, R> {
    type Error = R::Error;
}

impl<'a, R: Read> Read for Crc32Reader<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let written = self.reader.read(buf)?;
        self.hasher.update(buf);
        Ok(written)
    }
}
