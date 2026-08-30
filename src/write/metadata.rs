use embedded_io::Write;

pub trait WritableMetadataField {
    fn num_fields(&self) -> usize;
    fn write_to<W: Write>(self, writer: &mut W) -> Result<(), W::Error>;
}

