use embedded_io::Write;
use zerocopy::{Immutable, IntoBytes};

pub trait Writeable {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), W::Error>;
}

impl<T: IntoBytes + Immutable> Writeable for T {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), W::Error> {
        writer.write_all(self.as_bytes())
    }
}
