use embedded_io::Write;

pub trait WritableFrameField {
    const SIZE: usize;
    fn write_to<W: Write>(self, writer: &mut W) -> Result<(), W::Error>;
}
