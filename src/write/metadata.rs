use embedded_io::Write;

pub trait WritableMetadataField {
    fn num_fields(&self) -> usize;
    fn write_to<W: Write>(self, writer: &mut W) -> Result<(), W::Error>;
}

pub trait MetadataWriteState {}
pub struct Written {
    pub fields_remaining: usize,
}

impl Written {
    pub fn new(fields_remaining: usize) -> Self {
        Self { fields_remaining }
    }
}
impl MetadataWriteState for Written {}
pub struct NotWritten;
impl MetadataWriteState for NotWritten {}
