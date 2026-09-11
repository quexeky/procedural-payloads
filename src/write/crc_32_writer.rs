use crc32fast::Hasher;
use embedded_io::{ErrorType, Write};

pub struct Crc32Writer<W: Write + ?Sized> {
    hasher: Hasher,
    writer: W,
}
impl<W: Write + ?Sized> ErrorType for Crc32Writer<W> {
    type Error = W::Error;
}
impl<W: Write + ?Sized> Write for Crc32Writer<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let written = self.writer.write(buf)?;
        self.hasher.update(&buf[..written]);
        Ok(written)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.writer.flush()
    }
}

impl<W: Write> Crc32Writer<W> {
    pub fn new(writer: W, hasher: Hasher) -> Self {
        Self { hasher, writer }
    }
    pub fn into_inner(self) -> (Hasher, W) {
        (self.hasher, self.writer)
    }
    pub fn finish(self) -> u32 {
        self.hasher.finalize()
    }
    pub fn write_finish(mut self) -> Result<(), W::Error> {
        let hash = self.hasher.finalize();
        self.writer.write_all(&hash.to_be_bytes())
    }
}
