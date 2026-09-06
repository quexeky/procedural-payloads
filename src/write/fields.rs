use embedded_io::Write;
use zerocopy::{Immutable, IntoBytes};

pub trait WritableFrameField: IntoBytes + Immutable {
    const SIZE: usize;
    fn write_to<W: Write>(self, writer: &mut W) -> Result<(), W::Error>;
}
