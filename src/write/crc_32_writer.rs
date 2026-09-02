use crc32fast::Hasher;
use embedded_io::{ErrorType, Write};

pub struct Crc32Writer<'a, W: Write + ?Sized> {
    hasher: Hasher,
    writer: &'a mut W,
}
impl<W: Write + ?Sized> ErrorType for Crc32Writer<'_, W> {
    type Error = W::Error;
}
impl<W: Write + ?Sized> Write for Crc32Writer<'_, W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let written = self.writer.write(buf)?;
        self.hasher.update(&buf[..written]);
        Ok(written)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.writer.flush()
    }
}

impl<'a, W: Write + ?Sized> Crc32Writer<'a, W> {
    pub fn new(writer: &'a mut W, hasher: Hasher) -> Self {
        Self { hasher, writer }
    }
    pub fn into_inner(self) -> Hasher {
        self.hasher
    }
    pub fn finish(self) -> u32 {
        self.hasher.finalize()
    }
}
