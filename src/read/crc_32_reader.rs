use crc32fast::Hasher;
use embedded_io::{ErrorType, Read};

pub struct Crc32Reader<R: Read> {
    reader: R,
    hasher: Hasher,
}

impl<'a, R: Read> ErrorType for Crc32Reader<R> {
    type Error = R::Error;
}

impl<'a, R: Read> Read for Crc32Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let written = self.reader.read(buf)?;
        self.hasher.update(&buf[..written]);
        Ok(written)
    }
}

impl<'a, R: Read> Crc32Reader<R> {
    pub fn new(reader: R, hasher: Hasher) -> Self {
        Self { hasher, reader }
    }
    pub fn into_inner(self) -> Hasher {
        self.hasher
    }
    pub fn finish(self) -> u32 {
        self.hasher.finalize()
    }
}
