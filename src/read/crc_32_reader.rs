use crc32fast::Hasher;
use embedded_io::{ErrorType, Read};

pub struct Crc32Reader<R: Read> {
    reader: R,
    hasher: Hasher,
}

impl<R: Read> ErrorType for Crc32Reader<R> {
    type Error = R::Error;
}

impl<R: Read> Read for Crc32Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let read = self.reader.read(buf)?;
        self.hasher.update(&buf[..read]);
        Ok(read)
    }
}

impl<R: Read> Crc32Reader<R> {
    pub fn new(reader: R, hasher: Hasher) -> Self {
        Self { hasher, reader }
    }
    pub fn into_inner(self) -> (Hasher, R) {
        (self.hasher, self.reader)
    }
    pub fn finish(self) -> u32 {
        self.hasher.finalize()
    }
}
